#!/usr/bin/env python3
"""Generate static metadata for the pinned MediaPipe Face Landmarker v1 task.

This is a bounded, schema-driven FlatBuffer reader.  It never imports a model
runtime, unpickles data, or reads floating-point tensor values.  It reads only
bounded INT32 StridedSlice control payloads to validate strides.  All enum
names, union tags, option field names, and option defaults are read from the
pinned official TensorFlow Lite schema supplied with ``--schema``.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import re
import struct
import sys
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any


BUNDLE_BYTES = 3_758_596
BUNDLE_SHA256 = "64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff"
SCHEMA_REVISION = "e3388c0b93738d8d1a59211ffe2d0643da907291"
SCHEMA_URL = (
    "https://github.com/tensorflow/tflite-micro/blob/"
    f"{SCHEMA_REVISION}/tensorflow/lite/schema/schema.fbs"
)
SCHEMA_SHA256 = "e4739320658d85923286a2dedf1dd0c77387170c470ad7da2f3c22082edd9c4f"
ENTRY_NAMES = (
    "face_detector.tflite",
    "face_landmarks_detector.tflite",
    "geometry_pipeline_metadata_landmarks.binarypb",
    "face_blendshapes.tflite",
)
GRAPH_FILES = (
    ("Detector", "face_detector.tflite"),
    ("Landmarks", "face_landmarks_detector.tflite"),
    ("Blendshapes", "face_blendshapes.tflite"),
)
SUPPORTED_DTYPES = {"FLOAT32": "Float32", "FLOAT16": "Float16", "INT32": "Int32"}
DTYPE_BYTES = {"FLOAT32": 4, "FLOAT16": 2, "INT32": 4}
SUPPORTED_OPS = {
    "CONV_2D",
    "DEPTHWISE_CONV_2D",
    "MAX_POOL_2D",
    "ADD",
    "SUB",
    "MUL",
    "DIV",
    "RELU",
    "LOGISTIC",
    "PRELU",
    "NEG",
    "SQRT",
    "RSQRT",
    "SQUARED_DIFFERENCE",
    "DEQUANTIZE",
    "PAD",
    "RESHAPE",
    "CONCATENATION",
    "MEAN",
    "SUM",
    "TRANSPOSE",
    "STRIDED_SLICE",
}


class InspectError(Exception):
    """A malformed, unsupported, or inconsistent artifact."""


def fail(message: str) -> None:
    raise InspectError(message)


def strip_comments(source: str) -> str:
    return re.sub(r"//[^\n]*", "", source)


def balanced_block(source: str, pattern: str) -> str:
    match = re.search(pattern, source)
    if not match:
        fail(f"schema block not found: {pattern}")
    start = source.find("{", match.start())
    if start < 0:
        fail(f"schema block has no opening brace: {pattern}")
    depth = 0
    for pos in range(start, len(source)):
        if source[pos] == "{":
            depth += 1
        elif source[pos] == "}":
            depth -= 1
            if depth == 0:
                return source[start + 1 : pos]
    fail(f"unterminated schema block: {pattern}")


def parse_enum(source: str, name: str) -> dict[str, int]:
    body = balanced_block(source, rf"\benum\s+{re.escape(name)}\b")
    values: dict[str, int] = {}
    next_value = 0
    for raw in body.split(","):
        item = raw.strip()
        if not item:
            continue
        match = re.match(r"([A-Za-z_]\w*)\s*(?:=\s*(-?\d+))?$", item)
        if not match:
            continue
        key, explicit = match.groups()
        if explicit is not None:
            next_value = int(explicit)
        values[key] = next_value
        next_value += 1
    if not values:
        fail(f"schema enum has no values: {name}")
    return values


@dataclass(frozen=True)
class Field:
    name: str
    type_name: str
    default: str | None


def parse_table_fields(source: str, table_name: str) -> list[Field]:
    body = balanced_block(source, rf"\btable\s+{re.escape(table_name)}\b")
    fields: list[Field] = []
    for raw in body.split(";"):
        item = " ".join(raw.strip().split())
        if not item:
            continue
        match = re.match(r"([A-Za-z_]\w*)\s*:\s*([A-Za-z_]\w*(?:\s*\([^)]*\))?|\[[^]]+\])(?:\s*=\s*([^\s]+))?$", item)
        if match:
            fields.append(Field(match.group(1), match.group(2).replace(" ", ""), match.group(3)))
    return fields


class FB:
    """Small checked reader for the FlatBuffer primitives used here."""

    def __init__(self, data: bytes):
        self.data = data

    def check(self, pos: int, size: int, label: str) -> None:
        if pos < 0 or size < 0 or pos > len(self.data) - size:
            fail(f"{label}: out of bounds at {pos} size {size}")

    def unpack(self, fmt: str, pos: int, label: str) -> int:
        size = struct.calcsize(fmt)
        self.check(pos, size, label)
        return struct.unpack_from(fmt, self.data, pos)[0]

    def u8(self, pos: int, label: str) -> int:
        return self.unpack("<B", pos, label)

    def i8(self, pos: int, label: str) -> int:
        return self.unpack("<b", pos, label)

    def u16(self, pos: int, label: str) -> int:
        return self.unpack("<H", pos, label)

    def i16(self, pos: int, label: str) -> int:
        return self.unpack("<h", pos, label)

    def u32(self, pos: int, label: str) -> int:
        return self.unpack("<I", pos, label)

    def i32(self, pos: int, label: str) -> int:
        return self.unpack("<i", pos, label)

    def u64(self, pos: int, label: str) -> int:
        return self.unpack("<Q", pos, label)

    def table(self, pos: int, label: str) -> "Table":
        self.check(pos, 4, label)
        return Table(self, pos, label)

    def indirect(self, pos: int, label: str) -> int:
        value = self.u32(pos, label)
        target = pos + value
        self.check(target, 4, label)
        return target

    def string(self, pos: int, label: str) -> str:
        target = self.indirect(pos, label)
        length = self.u32(target, label)
        self.check(target + 4, length + 1, label)
        if self.data[target + 4 + length] != 0:
            fail(f"{label}: string has no NUL terminator")
        try:
            return self.data[target + 4 : target + 4 + length].decode("utf-8")
        except UnicodeDecodeError as exc:
            fail(f"{label}: invalid UTF-8: {exc}")


class Table:
    def __init__(self, fb: FB, pos: int, label: str):
        self.fb = fb
        self.pos = pos
        self.label = label
        self.vtable = pos - fb.i32(pos, label)
        fb.check(self.vtable, 4, label)
        vtable_size = fb.u16(self.vtable, label)
        object_size = fb.u16(self.vtable + 2, label)
        if vtable_size < 4 or object_size < 4:
            fail(f"{label}: invalid vtable")
        fb.check(self.vtable, vtable_size, label)
        fb.check(pos, object_size, label)
        self.fields = max(0, (vtable_size // 2) - 2)

    def address(self, index: int) -> int | None:
        if index < 0 or index >= self.fields:
            return None
        offset = self.fb.u16(self.vtable + 4 + index * 2, self.label)
        return self.pos + offset if offset else None

    def scalar(self, index: int, kind: str, default: int = 0) -> int:
        pos = self.address(index)
        if pos is None:
            return default
        readers = {"byte": self.fb.u8, "sbyte": self.fb.i8, "ubyte": self.fb.u8, "short": self.fb.i16, "ushort": self.fb.u16, "int": self.fb.i32, "uint": self.fb.u32, "long": self.fb.u64}
        if kind not in readers:
            fail(f"{self.label}: unsupported scalar kind {kind}")
        return readers[kind](pos, self.label)

    def object(self, index: int) -> "Table | None":
        pos = self.address(index)
        return None if pos is None else self.fb.table(self.fb.indirect(pos, self.label), self.label)

    def string(self, index: int, default: str = "") -> str:
        pos = self.address(index)
        return default if pos is None else self.fb.string(pos, self.label)

    def vector_meta(self, index: int) -> tuple[int, int] | None:
        pos = self.address(index)
        if pos is None:
            return None
        target = self.fb.indirect(pos, self.label)
        length = self.fb.u32(target, self.label)
        self.fb.check(target + 4, length, self.label)
        return target + 4, length

    def vector(self, index: int, kind: str) -> list[int]:
        meta = self.vector_meta(index)
        if meta is None:
            return []
        pos, length = meta
        sizes = {"byte": 1, "ubyte": 1, "short": 2, "ushort": 2, "int": 4, "uint": 4, "bool": 1}
        readers = {"byte": self.fb.i8, "ubyte": self.fb.u8, "short": self.fb.i16, "ushort": self.fb.u16, "int": self.fb.i32, "uint": self.fb.u32, "bool": self.fb.u8}
        if kind not in sizes:
            fail(f"{self.label}: unsupported vector kind {kind}")
        self.fb.check(pos, length * sizes[kind], self.label)
        return [readers[kind](pos + index * sizes[kind], self.label) for index in range(length)]

    def objects(self, index: int) -> list["Table"]:
        meta = self.vector_meta(index)
        if meta is None:
            return []
        pos, length = meta
        out = []
        for item in range(length):
            out.append(self.fb.table(self.fb.indirect(pos + item * 4, self.label), self.label))
        return out


def schema_types(schema: str) -> tuple[dict[str, int], dict[str, int], dict[str, list[Field]], dict[int, str]]:
    enums = {name: parse_enum(schema, name) for name in ("TensorType", "BuiltinOperator", "Padding", "ActivationFunctionType")}
    union = balanced_block(schema, r"\bunion\s+BuiltinOptions\b")
    union_names = [x.strip() for x in union.split(",") if x.strip()]
    option_tags = {index + 1: name for index, name in enumerate(union_names)}
    tables = {name: parse_table_fields(schema, name) for name in set(union_names)}
    return enums["TensorType"], enums["BuiltinOperator"], tables, option_tags


def schema_default(field: Field, enums: dict[str, int]) -> Any:
    if field.default is not None:
        raw = field.default
        if raw.lower() in {"true", "false"}:
            return raw.lower() == "true"
        if re.fullmatch(r"-?\d+", raw):
            return int(raw)
        return enums.get(field.type_name, {}).get(raw, raw)
    if field.type_name.startswith("["):
        return []
    if field.type_name in {"bool"}:
        return False
    return 0


def read_field(table: Table, index: int, field: Field, enums: dict[str, int]) -> Any:
    type_name = field.type_name
    default = schema_default(field, enums)
    if type_name.startswith("["):
        kind = type_name[1:-1]
        return table.vector(index, {"int": "int", "uint": "uint", "byte": "byte", "ubyte": "ubyte", "bool": "bool"}.get(kind, kind))
    if type_name == "bool":
        return bool(table.scalar(index, "byte", int(default)))
    if type_name in {"int", "uint", "byte", "ubyte", "short", "ushort", "long"}:
        return table.scalar(index, type_name, int(default))
    if type_name in enums:
        return enums[type_name].get(str(table.scalar(index, "byte", int(default))), table.scalar(index, "byte", int(default)))
    fail(f"{table.label}: unsupported schema option type {type_name}")


def option_values(table: Table | None, table_name: str, table_fields: dict[str, list[Field]], enums: dict[str, int], enum_names: dict[str, dict[int, str]]) -> dict[str, Any]:
    fields = table_fields.get(table_name, [])
    if table is None:
        if fields:
            return {field.name: schema_default(field, enums) for field in fields}
        return {}
    values: dict[str, Any] = {}
    for index, field in enumerate(fields):
        value = read_field(table, index, field, enums)
        if field.type_name in enum_names and isinstance(value, int):
            value = enum_names[field.type_name].get(value, f"UNKNOWN_{value}")
        values[field.name] = value
    return values


def product(shape: list[int], label: str) -> int:
    result = 1
    for value in shape:
        if value < 0:
            fail(f"{label}: negative shape dimension {value}")
        result *= value
        if result > 1 << 62:
            fail(f"{label}: shape product exceeds bound")
    return result


def read_zip(path: Path) -> tuple[bytes, dict[str, dict[str, Any]]]:
    data = path.read_bytes()
    if len(data) != BUNDLE_BYTES:
        fail(f"artifact byte length {len(data)} != pinned {BUNDLE_BYTES}")
    digest = hashlib.sha256(data).hexdigest()
    if digest != BUNDLE_SHA256:
        fail(f"artifact SHA-256 {digest} != pinned {BUNDLE_SHA256}")
    try:
        archive = zipfile.ZipFile(path)
    except (OSError, zipfile.BadZipFile) as exc:
        fail(f"artifact is not a ZIP: {exc}")
    infos = archive.infolist()
    if len(infos) != len(ENTRY_NAMES) or {info.filename for info in infos} != set(ENTRY_NAMES):
        fail("artifact ZIP entries do not exactly match pinned four-entry set")
    entries: dict[str, dict[str, Any]] = {}
    for info in infos:
        if info.compress_type != zipfile.ZIP_STORED:
            fail(f"{info.filename}: compressed ZIP entry is unsupported")
        if info.file_size != info.compress_size:
            fail(f"{info.filename}: stored ZIP size mismatch")
        header = info.header_offset
        if data[header : header + 4] != b"PK\x03\x04":
            fail(f"{info.filename}: invalid local ZIP header")
        name_len = struct.unpack_from("<H", data, header + 26)[0]
        extra_len = struct.unpack_from("<H", data, header + 28)[0]
        local_name = data[header + 30 : header + 30 + name_len].decode("utf-8")
        if local_name != info.filename:
            fail(f"{info.filename}: local header name mismatch")
        offset = header + 30 + name_len + extra_len
        if offset < 0 or offset + info.file_size > len(data):
            fail(f"{info.filename}: payload outside artifact")
        payload = data[offset : offset + info.file_size]
        entries[info.filename] = {
            "name": info.filename,
            "offset": offset,
            "byte_length": info.file_size,
            "crc32": f"{info.CRC:08x}",
            "sha256": hashlib.sha256(payload).hexdigest(),
            "payload": payload,
        }
    return data, entries


def parse_graph(name: str, filename: str, payload: bytes, entry_offset: int, entry_length: int, schema: str) -> dict[str, Any]:
    tensor_types, opcodes, option_tables, option_tags = schema_types(schema)
    enum_values = {
        "TensorType": tensor_types,
        "BuiltinOperator": opcodes,
        "Padding": parse_enum(schema, "Padding"),
        "ActivationFunctionType": parse_enum(schema, "ActivationFunctionType"),
    }
    enum_names = {name: {value: key for key, value in values.items()} for name, values in enum_values.items()}
    fb = FB(payload)
    root_pos = fb.u32(0, "Model root")
    model = fb.table(root_pos, "Model")
    version = model.scalar(0, "uint", 0)
    code_tables = model.objects(1)
    subgraphs = model.objects(2)
    buffers = model.objects(4)
    if len(subgraphs) != 1:
        fail(f"{filename}: expected one subgraph, got {len(subgraphs)}")
    if not buffers:
        fail(f"{filename}: model has no buffers")
    opcode_rows: list[dict[str, Any]] = []
    for index, code in enumerate(code_tables):
        deprecated_present = code.address(0) is not None
        deprecated = code.scalar(0, "sbyte", 0)
        custom = code.string(1)
        version_code = code.scalar(2, "int", 1)
        extended_present = code.address(3) is not None
        effective = code.scalar(3, "int", 0) if extended_present else (deprecated if deprecated_present else 0)
        if custom and effective == opcodes.get("CUSTOM"):
            opcode_name = custom
        else:
            reverse = {value: key for key, value in opcodes.items()}
            opcode_name = reverse.get(effective)
            if opcode_name is None:
                fail(f"{filename}: unknown builtin opcode {effective}")
        opcode_rows.append({"index": index, "code": effective, "name": opcode_name, "custom": custom, "version": version_code})
    buffer_rows: list[dict[str, int]] = []
    for index, buffer in enumerate(buffers):
        vector = buffer.vector_meta(0)
        if buffer.address(1) is not None or buffer.address(2) is not None:
            external_offset = buffer.scalar(1, "long", 0)
            external_size = buffer.scalar(2, "long", 0)
            if external_offset > 1 or external_size:
                fail(f"{filename}: external buffer {index} is unsupported")
        if vector is None:
            buffer_rows.append({"index": index, "local_offset": 0, "byte_length": 0})
        else:
            local_offset, byte_length = vector
            buffer_rows.append({"index": index, "local_offset": local_offset, "byte_length": byte_length})
    graph = subgraphs[0]
    tensor_tables = graph.objects(0)
    tensors: list[dict[str, Any]] = []
    for index, tensor in enumerate(tensor_tables):
        shape = tensor.vector(0, "int")
        dtype_value = tensor.scalar(1, "ubyte", 0)
        dtype_name = next((key for key, value in tensor_types.items() if value == dtype_value), None)
        if dtype_name not in SUPPORTED_DTYPES:
            fail(f"{filename}: tensor {index} has unsupported dtype {dtype_name or dtype_value}")
        buffer_index = tensor.scalar(2, "uint", 0)
        if buffer_index >= len(buffer_rows):
            fail(f"{filename}: tensor {index} buffer index {buffer_index} is out of range")
        buffer = buffer_rows[buffer_index]
        if buffer_index == 0 or buffer["byte_length"] == 0:
            offset = None
            byte_length = 0
        else:
            offset = entry_offset + buffer["local_offset"]
            byte_length = buffer["byte_length"]
            if offset < entry_offset or offset + byte_length > entry_offset + entry_length:
                fail(f"{filename}: tensor {index} payload range escapes its ZIP entry")
            expected = product(shape, f"{filename} tensor {index}") * DTYPE_BYTES[dtype_name]
            if expected != byte_length:
                fail(f"{filename}: tensor {index} buffer bytes {byte_length} != shape bytes {expected}")
        tensors.append({"index": index, "name": tensor.string(3), "shape": shape, "dtype": dtype_name, "storage_type": SUPPORTED_DTYPES[dtype_name], "buffer_index": buffer_index, "offset": offset, "byte_len": byte_length})
    graph_inputs = graph.vector(1, "int")
    graph_outputs = graph.vector(2, "int")
    for role, indices in (("input", graph_inputs), ("output", graph_outputs)):
        for index in indices:
            if index < 0 or index >= len(tensors):
                fail(f"{filename}: {role} tensor index {index} is out of range")
    operators = graph.objects(3)
    nodes: list[dict[str, Any]] = []
    produced: dict[int, int] = {}
    input_set = set(graph_inputs)
    for op_index, operator in enumerate(operators):
        opcode_index = operator.scalar(0, "uint", 0)
        if opcode_index >= len(opcode_rows):
            fail(f"{filename}: operator {op_index} opcode index {opcode_index} is out of range")
        opcode_name = opcode_rows[opcode_index]["name"]
        if opcode_name not in SUPPORTED_OPS:
            fail(f"{filename}: unsupported opcode {opcode_name}")
        inputs = operator.vector(1, "int")
        outputs = operator.vector(2, "int")
        if any(index < 0 for index in inputs):
            fail(f"{filename}: operator {op_index} has optional/negative input")
        if len(outputs) != 1:
            fail(f"{filename}: operator {op_index} has {len(outputs)} outputs; exactly one required")
        output = outputs[0]
        if output < 0 or output >= len(tensors):
            fail(f"{filename}: operator {op_index} output tensor {output} is out of range")
        for index in inputs:
            if index >= len(tensors):
                fail(f"{filename}: operator {op_index} input tensor {index} is out of range")
            producer = produced.get(index)
            if producer is not None and producer >= op_index:
                fail(f"{filename}: operator {op_index} input {index} is not topological")
            if producer is None and index not in input_set and tensors[index]["buffer_index"] == 0:
                fail(f"{filename}: operator {op_index} input {index} has no producer or constant buffer")
        if output in produced:
            fail(f"{filename}: tensor {output} has multiple producers")
        option_tag = operator.scalar(3, "ubyte", 0)
        option_table_name = option_tags.get(option_tag)
        if option_tag and option_table_name is None:
            fail(f"{filename}: unknown BuiltinOptions tag {option_tag}")
        option_table = operator.object(4)
        options = option_values(option_table, option_table_name or "", option_tables, enum_values, enum_names)
        if opcode_name == "STRIDED_SLICE":
            if len(inputs) != 4:
                fail(f"{filename}: StridedSlice operator {op_index} must have four inputs")
            rank = len(tensors[inputs[0]]["shape"])
            if rank <= 0 or rank > 30:
                fail(f"{filename}: StridedSlice operator {op_index} has unsupported input rank {rank}")
            masks = ("begin_mask", "end_mask", "ellipsis_mask", "new_axis_mask", "shrink_axis_mask")
            for mask_name in masks:
                mask = options.get(mask_name)
                if not isinstance(mask, int) or mask < 0 or mask & ~((1 << rank) - 1):
                    fail(f"{filename}: StridedSlice operator {op_index} has out-of-range {mask_name} {mask}")
            if options.get("ellipsis_mask", 0) != 0 or options.get("new_axis_mask", 0) != 0:
                fail(f"{filename}: StridedSlice operator {op_index} uses unsupported ellipsis/new-axis mask")
            for control_position, control_index in enumerate(inputs[1:4], start=1):
                control = tensors[control_index]
                if control["dtype"] != "INT32" or control["shape"] != [rank] or control["offset"] is None:
                    fail(f"{filename}: StridedSlice operator {op_index} control {control_position} is not an INT32 rank-length payload")
                control_start = control["offset"] - entry_offset
                control_end = control_start + control["byte_len"]
                if control_start < 0 or control_end > len(payload) or control["byte_len"] != rank * 4:
                    fail(f"{filename}: StridedSlice operator {op_index} control {control_position} payload range is invalid")
                values = struct.unpack_from("<" + "i" * rank, payload, control_start)
                if control_position == 3 and any(value <= 0 for value in values):
                    fail(f"{filename}: StridedSlice operator {op_index} has non-positive stride control")
        nodes.append({"index": op_index, "opcode_index": opcode_index, "op": opcode_name, "inputs": inputs, "output": output, "options": options, "options_type": option_table_name})
        produced[output] = op_index
    for index in graph_outputs:
        if index not in produced and index not in input_set and tensors[index]["buffer_index"] == 0:
            fail(f"{filename}: graph output {index} has no producer")
    # Distinct buffers may be intentionally referenced by multiple tensors, but
    # partially overlapping data ranges are an ambiguity in a fixed descriptor.
    ranges = [tensor for tensor in tensors if tensor["offset"] is not None and tensor["byte_len"]]
    for left_index, left in enumerate(ranges):
        left_end = left["offset"] + left["byte_len"]
        for right in ranges[left_index + 1 :]:
            right_end = right["offset"] + right["byte_len"]
            if max(left["offset"], right["offset"]) < min(left_end, right_end):
                if not (left["offset"] == right["offset"] and left["byte_len"] == right["byte_len"]):
                    fail(f"{filename}: overlapping non-identical tensor payloads")
    inventory = dict(sorted(collections.Counter(node["op"] for node in nodes).items()))
    return {"id": name, "file": filename, "schema_version": version, "name": graph.string(4), "tensor_count": len(tensors), "operator_count": len(nodes), "operator_codes": opcode_rows, "inputs": graph_inputs, "outputs": graph_outputs, "input_tensors": [tensors[index] for index in graph_inputs], "output_tensors": [tensors[index] for index in graph_outputs], "dtype_counts": dict(sorted(collections.Counter(tensor["dtype"] for tensor in tensors).items())), "operator_inventory": inventory, "tensors": tensors, "nodes": nodes}


def op_rust(node: dict[str, Any]) -> str:
    op = node["op"]
    options = node["options"]
    activation_map = {"NONE": "None", "RELU": "Relu", "RELU_N1_TO_1": "ReluN1To1", "RELU6": "Relu6", "TANH": "Tanh", "SIGN_BIT": "SignBit"}
    activation = lambda: activation_map[options["fused_activation_function"]]
    padding = lambda: options["padding"].title()
    if op == "CONV_2D":
        return f"Op::Conv2d {{ stride_h: {options['stride_h']}, stride_w: {options['stride_w']}, dilation_h: {options['dilation_h_factor']}, dilation_w: {options['dilation_w_factor']}, padding: Padding::{padding()}, activation: Activation::{activation()} }}"
    if op == "DEPTHWISE_CONV_2D":
        return f"Op::DepthwiseConv2d {{ stride_h: {options['stride_h']}, stride_w: {options['stride_w']}, dilation_h: {options['dilation_h_factor']}, dilation_w: {options['dilation_w_factor']}, depth_multiplier: {options['depth_multiplier']}, padding: Padding::{padding()}, activation: Activation::{activation()} }}"
    if op == "MAX_POOL_2D":
        return f"Op::MaxPool2d {{ stride_h: {options['stride_h']}, stride_w: {options['stride_w']}, filter_h: {options['filter_height']}, filter_w: {options['filter_width']}, padding: Padding::{padding()}, activation: Activation::{activation()} }}"
    if op in {"ADD", "SUB", "MUL", "DIV"}:
        return f"Op::{op.title()} {{ activation: Activation::{activation()} }}"
    if op in {"RELU", "LOGISTIC", "PRELU", "NEG", "SQRT", "RSQRT", "SQUARED_DIFFERENCE", "DEQUANTIZE", "PAD", "TRANSPOSE"}:
        names = {"RELU": "Relu", "LOGISTIC": "Logistic", "PRELU": "Prelu", "NEG": "Neg", "SQRT": "Sqrt", "RSQRT": "Rsqrt", "SQUARED_DIFFERENCE": "SquaredDifference", "DEQUANTIZE": "Dequantize", "PAD": "Pad", "TRANSPOSE": "Transpose"}
        return f"Op::{names[op]}"
    if op == "RESHAPE":
        # Newer TFLite writers leave ReshapeOptions.new_shape absent and pass
        # shape as its second tensor input; preserve schema default [] exactly.
        return f"Op::Reshape {{ new_shape: &{rust_slice(options.get('new_shape', []))} }}"
    if op == "CONCATENATION":
        return f"Op::Concat {{ axis: {options['axis']}, activation: Activation::{activation()} }}"
    if op in {"MEAN", "SUM"}:
        variant = "Mean" if op == "MEAN" else "Sum"
        return f"Op::{variant} {{ keep_dims: {str(options['keep_dims']).lower()} }}"
    if op == "STRIDED_SLICE":
        return "Op::StridedSlice { " + ", ".join(f"{key}: {options[key]}" for key in ("begin_mask", "end_mask", "ellipsis_mask", "new_axis_mask", "shrink_axis_mask")) + " }"
    fail(f"cannot render Rust op {op}")


def rust_slice(values: list[Any]) -> str:
    return "[" + ", ".join(str(value).lower() if isinstance(value, bool) else str(value) for value in values) + "]"


def rust_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=True)


def render_rust(graphs: list[dict[str, Any]], entries: dict[str, dict[str, Any]]) -> str:
    lines = ["//! Generated fixed metadata for the pinned MediaPipe Face Landmarker v1 task.\n", "//! Generated by tools/inspect-mediapipe-v1.py; no tensor values are embedded.\n", "\n", "use super::mediapipe_graph::{Activation, EntrySpec, GraphId, GraphSpec, Node, Op, Padding, StorageType, TensorSpec};\n", "\n", f"pub(crate) const BUNDLE_BYTES: usize = {BUNDLE_BYTES};\n", f"pub(crate) const BUNDLE_SHA256: &str = {rust_string(BUNDLE_SHA256)};\n", "\n", "pub(crate) const ENTRIES: &[EntrySpec] = &[\n"]
    for name in ENTRY_NAMES:
        entry = entries[name]
        lines.append(f"    EntrySpec {{ name: {rust_string(name)}, offset: {entry['offset']}, byte_len: {entry['byte_length']}, sha256: {rust_string(entry['sha256'])} }},\n")
    lines.append("];\n")
    graph_constants: list[tuple[str, str, str]] = []
    for graph in graphs:
        prefix = graph["id"].upper()
        tensor_name = f"{prefix}_TENSORS"
        node_name = f"{prefix}_NODES"
        lines.append(f"static {tensor_name}: &[TensorSpec] = &[\n")
        for tensor in graph["tensors"]:
            shape = rust_slice(tensor["shape"])
            offset = "None" if tensor["offset"] is None else f"Some({tensor['offset']})"
            lines.append(f"    TensorSpec {{ name: {rust_string(tensor['name'])}, shape: &{shape}, dtype: StorageType::{tensor['storage_type']}, offset: {offset}, byte_len: {tensor['byte_len']} }},\n")
        lines.append("];\n")
        lines.append(f"static {node_name}: &[Node] = &[\n")
        for node in graph["nodes"]:
            lines.append(f"    Node {{ op: {op_rust(node)}, inputs: &{rust_slice(node['inputs'])}, output: {node['output']} }},\n")
        lines.append("];\n")
        graph_constants.append((graph["id"], tensor_name, node_name))
    lines.append("pub(crate) static GRAPHS: &[GraphSpec] = &[\n")
    for graph, tensors, nodes in zip(graphs, [x[1] for x in graph_constants], [x[2] for x in graph_constants]):
        graph_id = graph["id"]
        lines.append(f"    GraphSpec {{ id: GraphId::{graph_id}, name: {rust_string(graph['name'])}, inputs: &{rust_slice(graph['inputs'])}, outputs: &{rust_slice(graph['outputs'])}, tensors: {tensors}, nodes: {nodes} }},\n")
    lines.append("];\n")
    return "".join(lines)


def json_graph(graph: dict[str, Any]) -> dict[str, Any]:
    return {key: graph[key] for key in ("id", "file", "schema_version", "name", "tensor_count", "operator_count", "operator_codes", "inputs", "outputs", "input_tensors", "output_tensors", "dtype_counts", "operator_inventory", "tensors", "nodes")}


def render_markdown(graphs: list[dict[str, Any]], entries: dict[str, dict[str, Any]]) -> str:
    out = ["# MediaPipe Face Landmarker v1 static artifact inventory", "", "Generated from exact artifact bytes by `tools/inspect-mediapipe-v1.py`. This is static metadata only: no inference, model import, conversion, or floating-point tensor values. Only bounded INT32 StridedSlice control payloads are read for stride validation.", "", "## Pinned provenance", "", f"- Artifact: `{BUNDLE_BYTES}` bytes, SHA-256 `{BUNDLE_SHA256}`.", f"- TFLite schema: [`schema.fbs` at revision {SCHEMA_REVISION}]({SCHEMA_URL}), SHA-256 `{SCHEMA_SHA256}`.", "- The local schema hash is checked before generation; all opcode names, tensor type names, BuiltinOptions union tags, option field names, enum names, and schema defaults are parsed from that source.", "- The model URL is the versioned MediaPipe Face Landmarker float16/1 task. Model-card/license evidence remains in `mediapipe-face-landmarker-v1-qualification.json`.", "", "## ZIP entries", "", "| name | absolute data offset | bytes | SHA-256 | CRC32 |", "| --- | ---: | ---: | --- | --- |"]
    for name in ENTRY_NAMES:
        entry = entries[name]
        out.append(f"| `{name}` | {entry['offset']} | {entry['byte_length']} | `{entry['sha256']}` | `{entry['crc32']}` |")
    out += ["", "## Graphs", "", "Operators are listed in FlatBuffer topological order. Tensor offsets are absolute offsets in full `.task` bytes; `None/0` means runtime tensor without payload."]
    for graph in graphs:
        out += ["", f"### {graph['id']} — `{graph['file']}`", "", f"- Schema version: `{graph['schema_version']}`; subgraph: `{graph['name']}`.", f"- Tensors: `{graph['tensor_count']}`; nodes: `{graph['operator_count']}`.", f"- Inputs: `{graph['inputs']}`; outputs: `{graph['outputs']}`.", f"- Dtypes: `{json.dumps(graph['dtype_counts'], sort_keys=True)}`.", f"- Corrected operator inventory: `{json.dumps(graph['operator_inventory'], sort_keys=True)}`.", "", "#### Operator codes", "", "| opcode table index | schema builtin code | name | version | custom code |", "| ---: | ---: | --- | ---: | --- |"]
        for opcode in graph["operator_codes"]:
            out.append(f"| {opcode['index']} | {opcode['code']} | `{opcode['name']}` | {opcode['version']} | `{opcode['custom']}` |")
        out += ["", "#### Tensors", "", "| index | name | dtype | shape | buffer index | absolute offset | bytes |", "| ---: | --- | --- | --- | ---: | ---: | ---: |"]
        for tensor in graph["tensors"]:
            shape = "[]" if not tensor["shape"] else "[" + ", ".join(str(x) for x in tensor["shape"]) + "]"
            offset = "None" if tensor["offset"] is None else str(tensor["offset"])
            out.append(f"| {tensor['index']} | `{tensor['name']}` | `{tensor['dtype']}` | `{shape}` | {tensor['buffer_index']} | {offset} | {tensor['byte_len']} |")
        out += ["", "#### Nodes", "", "| index | op | inputs | output | exact schema-derived options |", "| ---: | --- | --- | ---: | --- |"]
        for node in graph["nodes"]:
            out.append(f"| {node['index']} | `{node['op']}` | `{node['inputs']}` | {node['output']} | `{json.dumps(node['options'], sort_keys=True)}` |")
    out += ["", "## Reproducibility", "", "```text", "python3 tools/inspect-mediapipe-v1.py \\", "  --artifact /tmp/weights_faces_eyes/mediapipe-v1/face_landmarker-float16-v1.task \\", "  --schema /tmp/weights_faces_eyes/mediapipe-v1/schema.fbs \\", "  --rust-output crates/segment/src/mediapipe_inventory.rs \\", "  --json-output docs/models/mediapipe-face-landmarker-v1-qualification.json \\", "  --markdown-output docs/models/mediapipe-v1-artifact-inventory.md", "```", "", "Validation performed: exact artifact identity; exact stored ZIP entry set and payload ranges; every constant tensor range contained within its own ZIP entry; FlatBuffer bounds; supported dtype/op/option checks; no negative optional inputs; single-output nodes; topological producer order; shape-byte bounds; bounded INT32 StridedSlice controls with positive strides, rank-length shapes, zero ellipsis/new-axis masks, and in-range mask bits; and non-identical payload overlap rejection. Floating-point payload bytes are never read.", ""]
    out += ["After generation, format Rust metadata with repository settings: `rustfmt --edition 2024 crates/segment/src/mediapipe_inventory.rs`.", ""]
    return "\n".join(out)


def update_qualification(path: Path, data: bytes, entries: dict[str, dict[str, Any]], graphs: list[dict[str, Any]]) -> None:
    try:
        document = json.loads(path.read_text())
    except FileNotFoundError:
        document = {"artifact": {}, "inspection": {}, "license_data_evidence": {}, "qualification": {}}
    artifact = document.setdefault("artifact", {})
    artifact.update({"byte_length": len(data), "sha256": hashlib.sha256(data).hexdigest(), "format": "ZIP container; all four entries stored, no compression", "entries": [{key: entry[key] for key in ("name", "offset", "byte_length", "crc32", "sha256")} for entry in (entries[name] for name in ENTRY_NAMES)]})
    inspection = document.setdefault("inspection", {})
    inspection.update({"method": "bounded schema-driven FlatBuffer reader; only bounded INT32 StridedSlice control payloads read for stride validation; no inference, model/runtime imports, unpickle, or floating-point tensor values", "schema_source": SCHEMA_URL, "schema_revision": SCHEMA_REVISION, "schema_sha256": SCHEMA_SHA256, "weight_values_recorded": False, "graphs": [json_graph(graph) for graph in graphs]})
    document["qualification"] = {**document.get("qualification", {}), "result": "none qualified", "status": "static graph metadata complete; conversion, execution, parity, benchmark, and held-out accuracy remain unperformed"}
    path.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact", required=True, type=Path)
    parser.add_argument("--schema", required=True, type=Path)
    parser.add_argument("--rust-output", required=True, type=Path)
    parser.add_argument("--json-output", required=True, type=Path)
    parser.add_argument("--markdown-output", required=True, type=Path)
    args = parser.parse_args()
    try:
        schema_bytes = args.schema.read_bytes()
        schema_hash = hashlib.sha256(schema_bytes).hexdigest()
        if schema_hash != SCHEMA_SHA256:
            fail(f"schema SHA-256 {schema_hash} != pinned official {SCHEMA_SHA256}")
        schema = strip_comments(schema_bytes.decode("utf-8"))
        data, entries = read_zip(args.artifact)
        graphs = [parse_graph(name, filename, entries[filename]["payload"], entries[filename]["offset"], entries[filename]["byte_length"], schema) for name, filename in GRAPH_FILES]
        args.rust_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
        args.rust_output.write_text(render_rust(graphs, entries))
        update_qualification(args.json_output, data, entries, graphs)
        args.markdown_output.write_text(render_markdown(graphs, entries))
        print(json.dumps({"artifact_bytes": len(data), "artifact_sha256": hashlib.sha256(data).hexdigest(), "graphs": [{"id": graph["id"], "tensors": graph["tensor_count"], "nodes": graph["operator_count"], "operator_inventory": graph["operator_inventory"]} for graph in graphs]}, indent=2))
        return 0
    except (OSError, UnicodeDecodeError, struct.error, zipfile.BadZipFile, InspectError) as exc:
        print(f"inspect-mediapipe-v1: error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
