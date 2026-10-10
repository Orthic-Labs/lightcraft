#!/usr/bin/env python3
"""Extract pinned Face Landmarker image metadata without reading weights.

This inspector verifies all pinned bytes before parsing. It reads TFLite graph
descriptors plus the metadata buffer only; constant model buffers are hashed by
the ZIP receipt but never interpreted as tensor values.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import io
import json
import math
import re
import struct
import sys
import zipfile
import zlib
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

_INSPECTOR = Path(__file__).with_name("inspect-mediapipe-v1.py")
_SPEC = importlib.util.spec_from_file_location("mediapipe_checked_fb", _INSPECTOR)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError("cannot load sibling checked FlatBuffer reader")
_FBMOD = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _FBMOD
_SPEC.loader.exec_module(_FBMOD)

FB = _FBMOD.FB
Field = _FBMOD.Field
balanced_block = _FBMOD.balanced_block
parse_enum = _FBMOD.parse_enum


ARTIFACT_BYTES = 3_758_596
ARTIFACT_SHA256 = "64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff"
ARTIFACT_URL = "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task"
TFLITE_SCHEMA_REVISION = "e3388c0b93738d8d1a59211ffe2d0643da907291"
TFLITE_SCHEMA_SHA256 = "e4739320658d85923286a2dedf1dd0c77387170c470ad7da2f3c22082edd9c4f"
TFLITE_SCHEMA_BYTES = 47_962
TFLITE_SCHEMA_URL = f"https://github.com/tensorflow/tflite-micro/blob/{TFLITE_SCHEMA_REVISION}/tensorflow/lite/schema/schema.fbs"
METADATA_SCHEMA_REVISION = "458200ffced12a9db596ddf1cce3380ea05f71fb"
METADATA_SCHEMA_SHA256 = "54f0cc99a87d56045bc32ad177bf5d02a29836416493cab3e98a06057b60797b"
METADATA_SCHEMA_BYTES = 28_030
METADATA_SCHEMA_URL = f"https://raw.githubusercontent.com/google-ai-edge/mediapipe/{METADATA_SCHEMA_REVISION}/mediapipe/tasks/metadata/metadata_schema.fbs"
ENTRY_NAMES = (
    "face_detector.tflite",
    "face_landmarks_detector.tflite",
    "geometry_pipeline_metadata_landmarks.binarypb",
    "face_blendshapes.tflite",
)
METADATA_MAX_BYTES = 256 * 1024
SCHEMA_MAX_BYTES = 1024 * 1024
STRING_MAX_BYTES = 512
VECTOR_MAX_LENGTH = 32
GRAPH_VECTOR_MAX_LENGTH = 1_000


class MetadataError(Exception):
    """Malformed, unpinned, or unsupported input."""


def fail(message: str) -> None:
    raise MetadataError(message)


def read_bounded(path: Path, maximum: int, label: str) -> bytes:
    with path.open("rb") as source:
        data = source.read(maximum + 1)
    if len(data) > maximum:
        fail(f"{label}: exceeds bounded read of {maximum} bytes")
    return data


def verify_hash(data: bytes, expected: str, label: str) -> None:
    actual = hashlib.sha256(data).hexdigest()
    if actual != expected:
        fail(f"{label}: SHA-256 {actual} != pinned {expected}")


def schema_fields(schema: str, table_name: str, unions: set[str] | None = None) -> dict[str, int]:
    """Derive field indices, expanding FlatBuffers union type/object pairs."""
    schema = _FBMOD.strip_comments(schema)
    unions = unions or set()
    body = balanced_block(schema, rf"\btable\s+{re.escape(table_name)}\b")
    fields: list[Field] = []
    for raw in body.split(";"):
        item = " ".join(raw.strip().split())
        if not item:
            continue
        match = re.match(
            r"([A-Za-z_]\w*)\s*:\s*([A-Za-z_]\w*(?:\s*\([^)]*\))?|\[[^]]+\])"
            r"(?:\s*\([^)]*\))?(?:\s*=\s*([^\s]+))?$",
            item,
        )
        if match:
            fields.append(Field(match.group(1), match.group(2).replace(" ", ""), match.group(3)))
    expanded: list[Field] = []
    for field in fields:
        if field.type_name in unions:
            expanded.append(Field(f"{field.name}_type", "ubyte", None))
        expanded.append(field)
    return {field.name: index for index, field in enumerate(expanded)}


def union_members(schema: str, union_name: str) -> dict[int, str]:
    schema = _FBMOD.strip_comments(schema)
    body = balanced_block(schema, rf"\bunion\s+{union_name}\b")
    members = [item.strip() for item in body.split(",") if item.strip()]
    return {index + 1: member for index, member in enumerate(members)}


def bounded_string(table: Any, index: int, label: str) -> str:
    position = table.address(index)
    if position is None:
        return ""
    target = table.fb.indirect(position, label)
    length = table.fb.u32(target, label)
    if length > STRING_MAX_BYTES:
        fail(f"{label}: string length {length} exceeds {STRING_MAX_BYTES}")
    table.fb.check(target + 4, length + 1, label)
    if table.fb.data[target + 4 + length] != 0:
        fail(f"{label}: string is not NUL terminated")
    try:
        return table.fb.data[target + 4 : target + 4 + length].decode("utf-8")
    except UnicodeDecodeError as exc:
        fail(f"{label}: invalid UTF-8: {exc}")


def bounded_objects(table: Any, index: int, label: str, maximum: int = VECTOR_MAX_LENGTH) -> list[Any]:
    meta = table.vector_meta(index)
    if meta is None:
        return []
    position, length = meta
    if length > maximum:
        fail(f"{label}: vector length {length} exceeds {maximum}")
    return [table.fb.table(table.fb.indirect(position + item * 4, label), label) for item in range(length)]


def bounded_int_vector(table: Any, index: int, label: str, maximum: int) -> list[int]:
    meta = table.vector_meta(index)
    if meta is None:
        return []
    position, length = meta
    if length > maximum:
        fail(f"{label}: vector length {length} exceeds {maximum}")
    return table.vector(index, "int")


def bounded_float_vector(table: Any, index: int, label: str) -> list[float]:
    meta = table.vector_meta(index)
    if meta is None:
        return []
    position, length = meta
    if length > 3:
        fail(f"{label}: float vector length {length} exceeds 3")
    table.fb.check(position, length * 4, label)
    values = [struct.unpack_from("<f", table.fb.data, position + 4 * item)[0] for item in range(length)]
    if any(not math.isfinite(value) for value in values):
        fail(f"{label}: non-finite normalization value")
    return values


def parse_zip(data: bytes) -> dict[str, dict[str, Any]]:
    """Parse exactly one verified owned artifact buffer."""
    try:
        archive = zipfile.ZipFile(io.BytesIO(data), "r")
    except (OSError, zipfile.BadZipFile) as exc:
        fail(f"artifact is not ZIP: {exc}")
    infos = archive.infolist()
    if len(infos) != len(ENTRY_NAMES) or {info.filename for info in infos} != set(ENTRY_NAMES):
        fail("artifact ZIP entries do not exactly match pinned four-entry set")
    entries: dict[str, dict[str, Any]] = {}
    for info in infos:
        if info.compress_type != zipfile.ZIP_STORED or info.file_size != info.compress_size:
            fail(f"{info.filename}: expected stored ZIP entry")
        header = info.header_offset
        if header < 0 or header + 30 > len(data) or data[header : header + 4] != b"PK\x03\x04":
            fail(f"{info.filename}: invalid local ZIP header")
        name_length = struct.unpack_from("<H", data, header + 26)[0]
        extra_length = struct.unpack_from("<H", data, header + 28)[0]
        name_end = header + 30 + name_length
        payload_offset = name_end + extra_length
        if name_end > len(data) or payload_offset > len(data) or payload_offset + info.file_size > len(data):
            fail(f"{info.filename}: payload outside artifact")
        local_name = data[header + 30 : name_end].decode("utf-8")
        if local_name != info.filename:
            fail(f"{info.filename}: local header name mismatch")
        payload = data[payload_offset : payload_offset + info.file_size]
        crc32 = zlib.crc32(payload) & 0xFFFFFFFF
        if crc32 != info.CRC:
            fail(f"{info.filename}: CRC mismatch")
        entries[info.filename] = {
            "bytes": info.file_size,
            "crc32": f"{crc32:08x}",
            "sha256": hashlib.sha256(payload).hexdigest(),
            "payload": payload,
        }
    return entries


def parse_tflite(payload: bytes, filename: str, tflite_schema: str) -> dict[str, Any]:
    enums = parse_enum(tflite_schema, "TensorType")
    model_fields = schema_fields(tflite_schema, "Model")
    subgraph_fields = schema_fields(tflite_schema, "SubGraph")
    tensor_fields = schema_fields(tflite_schema, "Tensor")
    buffer_fields = schema_fields(tflite_schema, "Buffer")
    metadata_fields = schema_fields(tflite_schema, "Metadata")
    fb = FB(payload)
    root = fb.table(fb.u32(0, f"{filename} Model root"), f"{filename} Model")
    subgraphs = bounded_objects(root, model_fields["subgraphs"], f"{filename} subgraphs")
    if len(subgraphs) != 1:
        fail(f"{filename}: expected one subgraph")
    buffers = bounded_objects(root, model_fields["buffers"], f"{filename} buffers", GRAPH_VECTOR_MAX_LENGTH)
    if not buffers:
        fail(f"{filename}: no buffers")
    metadata_rows = bounded_objects(root, model_fields["metadata"], f"{filename} metadata")
    subgraph = subgraphs[0]
    tensors = bounded_objects(subgraph, subgraph_fields["tensors"], f"{filename} tensors", GRAPH_VECTOR_MAX_LENGTH)
    inputs = bounded_int_vector(subgraph, subgraph_fields["inputs"], f"{filename} inputs", VECTOR_MAX_LENGTH)
    outputs = bounded_int_vector(subgraph, subgraph_fields["outputs"], f"{filename} outputs", VECTOR_MAX_LENGTH)
    def tensor_descriptor(index: int, role: str) -> dict[str, Any]:
        if index < 0 or index >= len(tensors):
            fail(f"{filename}: {role} tensor index out of range")
        tensor = tensors[index]
        dtype_value = tensor.scalar(tensor_fields["type"], "ubyte", 0)
        dtype = next((name for name, value in enums.items() if value == dtype_value), f"UNKNOWN_{dtype_value}")
        return {
            "index": index,
            "name": bounded_string(tensor, tensor_fields["name"], f"{filename} tensor name"),
            "shape": bounded_int_vector(tensor, tensor_fields["shape"], f"{filename} tensor shape", 8),
            "dtype": dtype,
        }
    input_descriptors = [tensor_descriptor(index, "input") for index in inputs]
    output_descriptors = [tensor_descriptor(index, "output") for index in outputs]
    metadata_blob: bytes | None = None
    for metadata in metadata_rows:
        name = bounded_string(metadata, metadata_fields["name"], f"{filename} metadata name")
        if name != "TFLITE_METADATA":
            continue
        buffer_index = metadata.scalar(metadata_fields["buffer"], "uint", 0)
        if buffer_index >= len(buffers):
            fail(f"{filename}: metadata buffer index out of range")
        vector = buffers[buffer_index].vector_meta(buffer_fields["data"])
        if vector is None:
            fail(f"{filename}: empty TFLITE_METADATA buffer")
        position, length = vector
        if length > METADATA_MAX_BYTES:
            fail(f"{filename}: metadata buffer exceeds {METADATA_MAX_BYTES} bytes")
        buffers[buffer_index].fb.check(position, length, f"{filename} metadata buffer")
        metadata_blob = payload[position : position + length]
        break
    if metadata_blob is None:
        fail(f"{filename}: TFLITE_METADATA not found")
    return {
        "filename": filename,
        "input_tensors": input_descriptors,
        "output_tensors": output_descriptors,
        "metadata_blob": metadata_blob,
    }


def parse_model_metadata(graph: dict[str, Any], metadata_schema: str) -> dict[str, Any]:
    process_unions = union_members(metadata_schema, "ProcessUnitOptions")
    content_unions = union_members(metadata_schema, "ContentProperties")
    model_fields = schema_fields(metadata_schema, "ModelMetadata")
    subgraph_fields = schema_fields(metadata_schema, "SubGraphMetadata")
    tensor_fields = schema_fields(metadata_schema, "TensorMetadata")
    process_fields = schema_fields(metadata_schema, "ProcessUnit", {"ProcessUnitOptions"})
    content_fields = schema_fields(metadata_schema, "Content", {"ContentProperties"})
    image_fields = schema_fields(metadata_schema, "ImageProperties")
    normalization_fields = schema_fields(metadata_schema, "NormalizationOptions")
    color_space = {value: name for name, value in parse_enum(_FBMOD.strip_comments(metadata_schema), "ColorSpaceType").items()}
    fb = FB(graph["metadata_blob"])
    root = fb.table(fb.u32(0, f"{graph['filename']} ModelMetadata root"), f"{graph['filename']} ModelMetadata")
    subgraphs = bounded_objects(root, model_fields["subgraph_metadata"], f"{graph['filename']} metadata subgraphs")
    if len(subgraphs) != 1:
        fail(f"{graph['filename']}: expected one metadata subgraph")
    subgraph = subgraphs[0]
    input_meta = bounded_objects(subgraph, subgraph_fields["input_tensor_metadata"], f"{graph['filename']} input metadata")
    output_meta = bounded_objects(subgraph, subgraph_fields["output_tensor_metadata"], f"{graph['filename']} output metadata")
    if len(input_meta) != len(graph["input_tensors"]) or len(output_meta) != len(graph["output_tensors"]):
        fail(f"{graph['filename']}: metadata tensor counts do not match graph")
    def tensor_metadata(table: Any, descriptor: dict[str, Any], role: str) -> dict[str, Any]:
        name = bounded_string(table, tensor_fields["name"], f"{graph['filename']} {role} metadata name")
        content = table.object(tensor_fields["content"])
        content_name = "NONE"
        rgb_content: str | None = None
        if content is not None:
            content_type = content.scalar(content_fields["content_properties_type"], "ubyte", 0)
            content_name = content_unions.get(content_type, f"UNKNOWN_{content_type}")
            if content_name == "ImageProperties":
                image = content.object(content_fields["content_properties"])
                if image is None:
                    fail(f"{graph['filename']}: ImageProperties object missing")
                color_value = image.scalar(image_fields["color_space"], "byte", 0)
                rgb_content = color_space.get(color_value, f"UNKNOWN_{color_value}")
        means: list[float] = []
        stds: list[float] = []
        units = bounded_objects(table, tensor_fields["process_units"], f"{graph['filename']} {role} process units")
        for unit in units:
            unit_type = unit.scalar(process_fields["options_type"], "ubyte", 0)
            unit_name = process_unions.get(unit_type, f"UNKNOWN_{unit_type}")
            if unit_name != "NormalizationOptions":
                continue
            normalization = unit.object(process_fields["options"])
            if normalization is None:
                fail(f"{graph['filename']}: normalization object missing")
            means = bounded_float_vector(normalization, normalization_fields["mean"], f"{graph['filename']} mean")
            stds = bounded_float_vector(normalization, normalization_fields["std"], f"{graph['filename']} std")
            if not means or not stds or len(means) != len(stds):
                fail(f"{graph['filename']}: invalid normalization vector lengths")
            if any(value == 0.0 for value in stds):
                fail(f"{graph['filename']}: zero normalization std")
        if not means or not stds:
            fail(f"{graph['filename']}: normalization metadata missing")
        return {
            "metadata_name": name,
            "model_tensor_name": descriptor["name"],
            "shape": descriptor["shape"],
            "dtype": descriptor["dtype"],
            "content_properties": content_name,
            "rgb_content": rgb_content,
            "normalization": {"formula": "(x - mean) / std", "mean": means, "std": stds},
        }
    model_name = bounded_string(root, model_fields["name"], f"{graph['filename']} model metadata name")
    return {
        "model_name": model_name,
        "inputs": [
            {"order": order, **tensor_metadata(table, descriptor, "input")}
            for order, (table, descriptor) in enumerate(zip(input_meta, graph["input_tensors"]))
        ],
        "outputs": [
            {"order": order, "metadata_name": bounded_string(table, tensor_fields["name"], f"{graph['filename']} output metadata name"), "model_tensor_name": descriptor["name"], "shape": descriptor["shape"], "dtype": descriptor["dtype"]}
            for order, (table, descriptor) in enumerate(zip(output_meta, graph["output_tensors"]))
        ],
    }


def inspect(args: argparse.Namespace) -> dict[str, Any]:
    artifact = read_bounded(args.artifact, ARTIFACT_BYTES, "artifact")
    if len(artifact) != ARTIFACT_BYTES:
        fail(f"artifact byte length {len(artifact)} != pinned {ARTIFACT_BYTES}")
    verify_hash(artifact, ARTIFACT_SHA256, "artifact")
    tflite_schema_bytes = read_bounded(args.tflite_schema, SCHEMA_MAX_BYTES, "TFLite schema")
    if len(tflite_schema_bytes) != TFLITE_SCHEMA_BYTES:
        fail(f"TFLite schema byte length {len(tflite_schema_bytes)} != pinned {TFLITE_SCHEMA_BYTES}")
    verify_hash(tflite_schema_bytes, TFLITE_SCHEMA_SHA256, "TFLite schema")
    metadata_schema_bytes = read_bounded(args.metadata_schema, SCHEMA_MAX_BYTES, "metadata schema")
    if len(metadata_schema_bytes) != METADATA_SCHEMA_BYTES:
        fail(f"metadata schema byte length {len(metadata_schema_bytes)} != pinned {METADATA_SCHEMA_BYTES}")
    verify_hash(metadata_schema_bytes, METADATA_SCHEMA_SHA256, "metadata schema")
    try:
        tflite_schema = tflite_schema_bytes.decode("utf-8")
        metadata_schema = metadata_schema_bytes.decode("utf-8")
    except UnicodeDecodeError as exc:
        fail(f"schema UTF-8 decode failed: {exc}")
    entries = parse_zip(artifact)
    graphs: list[dict[str, Any]] = []
    for filename in ("face_detector.tflite", "face_landmarks_detector.tflite"):
        graph = parse_tflite(entries[filename]["payload"], filename, tflite_schema)
        graph["metadata"] = parse_model_metadata(graph, metadata_schema)
        del graph["metadata_blob"]
        graphs.append(graph)
    expected = {
        "face_detector.tflite": ([127.5], [127.5]),
        "face_landmarks_detector.tflite": ([0.0], [255.0]),
    }
    for graph in graphs:
        means = graph["metadata"]["inputs"][0]["normalization"]["mean"]
        stds = graph["metadata"]["inputs"][0]["normalization"]["std"]
        if (means, stds) != expected[graph["filename"]]:
            fail(f"{graph['filename']}: unexpected pinned normalization")
        if graph["metadata"]["inputs"][0]["rgb_content"] != "RGB":
            fail(f"{graph['filename']}: expected RGB image content metadata")
    output = {
        "contract": "MediaPipe Face Landmarker v1 image metadata",
        "inspection": "static metadata only; no inference or float weight reads",
        "artifact": {"url": ARTIFACT_URL, "bytes": len(artifact), "sha256": ARTIFACT_SHA256, "entries": [{"name": name, "bytes": entries[name]["bytes"], "crc32": entries[name]["crc32"], "sha256": entries[name]["sha256"]} for name in ENTRY_NAMES]},
        "schemas": {
            "tflite": {"url": TFLITE_SCHEMA_URL, "revision": TFLITE_SCHEMA_REVISION, "bytes": len(tflite_schema_bytes), "sha256": TFLITE_SCHEMA_SHA256},
            "metadata": {"url": METADATA_SCHEMA_URL, "revision": METADATA_SCHEMA_REVISION, "bytes": len(metadata_schema_bytes), "sha256": METADATA_SCHEMA_SHA256},
        },
        "models": graphs,
    }
    return output


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact", required=True, type=Path)
    parser.add_argument("--tflite-schema", required=True, type=Path)
    parser.add_argument("--metadata-schema", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    try:
        document = inspect(args)
        serialized = json.dumps(document, indent=2, sort_keys=True) + "\n"
        if len(serialized.encode("utf-8")) > 1024 * 1024:
            fail("serialized output exceeds 1 MiB")
        with args.out.open("x", encoding="utf-8") as destination:
            destination.write(serialized)
    except (_FBMOD.InspectError, KeyError) as exc:
        parser.error(f"schema mismatch: {exc}")
    except (MetadataError, OSError, struct.error, UnicodeError, ValueError, zipfile.BadZipFile) as exc:
        parser.error(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
