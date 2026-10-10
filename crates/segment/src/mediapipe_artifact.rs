//! Immutable, identity-checked loader for the pinned MediaPipe Face Landmarker bundle.
//!
//! This module deliberately consumes only static metadata supplied by
//! [`crate::mediapipe_inventory`].  It never opens the zip container or parses FlatBuffers at
//! runtime: the complete artifact is hashed first, then every metadata range is checked against
//! that exact byte buffer before constants are materialised.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use candle_core::{Device, Tensor};

use crate::mediapipe_graph::{GraphId, GraphSpec, LoadedGraph, Node, Op, Padding, StorageType, TensorSpec, Value};
use crate::mediapipe_inventory::{BUNDLE_BYTES as INVENTORY_BUNDLE_BYTES, BUNDLE_SHA256 as INVENTORY_BUNDLE_SHA256, ENTRIES, GRAPHS};
use crate::{Error, Result};

/// Exact byte length of the pinned Face Landmarker v1 bundle.
pub const BUNDLE_BYTES: usize = INVENTORY_BUNDLE_BYTES;
/// SHA-256 of the exact pinned Face Landmarker v1 bundle.
pub const BUNDLE_SHA256: &str = INVENTORY_BUNDLE_SHA256;

const GRAPH_COUNT: usize = 3;
const MAX_TENSORS: usize = 1_000;
const MAX_NODES: usize = 1_000;
const MAX_TOTAL_TENSORS: usize = 3_000;
const MAX_TOTAL_NODES: usize = 3_000;
const MAX_ELEMENTS: usize = 16_000_000;
const MAX_DECODED_ELEMENTS: usize = 48_000_000;
const MAX_PAYLOAD: usize = 3_800_000;
const MAX_RANK: usize = 8;

/// Loaded graph constants.  Graph descriptors remain static and are borrowed from inventory;
/// only immutable constant values are owned by this bundle.
pub struct Bundle {
    pub(crate) graphs: Vec<LoadedGraph>,
}

impl Bundle {
    pub(crate) fn graph(&self, id: GraphId) -> Result<&LoadedGraph> {
        self.graphs.iter().find(|graph| graph.spec.id == id).ok_or_else(|| Error::Model(format!("MediaPipe graph {id:?} is not present in bundle")))
    }
}

/// Read at most one byte past the pinned artifact size, then perform one identity check.
pub fn load_file(path: &Path, device: &Device) -> Result<Bundle> {
    let file = File::open(path).map_err(|error| Error::Model(format!("{}: {error}", path.display())))?;
    let max_read = u64::try_from(BUNDLE_BYTES)
        .ok()
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or_else(|| Error::Model("MediaPipe artifact byte limit overflow".into()))?;
    let mut bytes = Vec::with_capacity(BUNDLE_BYTES.saturating_add(1));
    file.take(max_read).read_to_end(&mut bytes).map_err(|error| Error::Model(format!("{}: {error}", path.display())))?;
    load_verified_bytes(bytes, device)
}

/// Load constants from an owned, exactly identified artifact.
pub fn load_verified_bytes(bytes: Vec<u8>, device: &Device) -> Result<Bundle> {
    verify_artifact_bytes(&bytes)?;
    validate_inventory(&bytes)?;
    validate_graphs(&bytes)?;

    // Decode all values into host memory first.  This also makes finite-value checking independent
    // of Candle/device backends; no device allocation occurs until every descriptor is validated.
    let mut decoded: Vec<Option<ValueData>> = Vec::new();
    let mut ranges: Vec<(usize, usize, StorageType, usize)> = Vec::new();
    let mut decoded_elements = 0usize;
    for graph in GRAPHS {
        for tensor in graph.tensors {
            let Some(offset) = tensor.offset else {
                decoded.push(None);
                continue;
            };
            let end = offset.checked_add(tensor.byte_len).ok_or_else(|| Error::Model(format!("MediaPipe tensor {} range overflow", tensor.name)))?;
            let raw = bytes.get(offset..end).ok_or_else(|| Error::Model(format!("MediaPipe tensor {} range is outside artifact", tensor.name)))?;
            let cache = ranges.iter().position(|(cached_offset, cached_len, cached_dtype, _)| {
                *cached_offset == offset && *cached_len == tensor.byte_len && *cached_dtype == tensor.dtype
            });
            if let Some(index) = cache {
                let cached_index = ranges[index].3;
                decoded.push(decoded.get(cached_index).cloned().flatten());
                continue;
            }
            let elements =
                element_count(tensor.shape).ok_or_else(|| Error::Model(format!("MediaPipe tensor {} element count overflows", tensor.name)))?;
            decoded_elements =
                decoded_elements.checked_add(elements).ok_or_else(|| Error::Model("MediaPipe decoded element count overflows".into()))?;
            if decoded_elements > MAX_DECODED_ELEMENTS {
                return Err(Error::Model("MediaPipe decoded constants exceed element bound".into()));
            }
            let value = decode_value(tensor, raw)?;
            ranges.push((offset, tensor.byte_len, tensor.dtype, decoded.len()));
            decoded.push(Some(value));
        }
    }

    let mut cursor = 0usize;
    let mut materialized: Vec<(usize, usize, StorageType, &'static [usize], Value)> = Vec::new();
    let mut graphs = Vec::with_capacity(GRAPHS.len());
    for graph in GRAPHS {
        let mut constants = Vec::with_capacity(graph.tensors.len());
        for tensor in graph.tensors {
            let value = if tensor.offset.is_some() {
                let cached = materialized.iter().find(|(offset, byte_len, dtype, shape, _)| {
                    Some(*offset) == tensor.offset && *byte_len == tensor.byte_len && *dtype == tensor.dtype && *shape == tensor.shape
                });
                if let Some((_, _, _, _, value)) = cached {
                    Some(value.clone())
                } else {
                    let offset = tensor.offset.ok_or_else(|| Error::Model(format!("MediaPipe constant {} has no byte offset", tensor.name)))?;
                    let value = decoded
                        .get(cursor)
                        .cloned()
                        .flatten()
                        .map(|value| materialize_value(value, tensor, device))
                        .transpose()?
                        .ok_or_else(|| Error::Model(format!("MediaPipe constant {} is missing decoded data", tensor.name)))?;
                    materialized.push((offset, tensor.byte_len, tensor.dtype, tensor.shape, value.clone()));
                    Some(value)
                }
            } else {
                None
            };
            constants.push(value);
            cursor = cursor.checked_add(1).ok_or_else(|| Error::Model("MediaPipe tensor cursor overflow".into()))?;
        }
        graphs.push(LoadedGraph { spec: graph, constants });
    }
    Ok(Bundle { graphs })
}

#[derive(Clone, Debug)]
enum ValueData {
    Float(Arc<[f32]>),
    Int(Arc<[i32]>),
}

fn materialize_value(value: ValueData, tensor: &TensorSpec, device: &Device) -> Result<Value> {
    match value {
        ValueData::Float(values) => Ok(Value::Float(Tensor::from_vec(values.to_vec(), tensor.shape, device)?)),
        ValueData::Int(values) => Ok(Value::Int { values: values.to_vec(), shape: tensor.shape.to_vec() }),
    }
}

fn decode_value(tensor: &TensorSpec, raw: &[u8]) -> Result<ValueData> {
    match tensor.dtype {
        StorageType::Float32 => {
            let values = raw
                .chunks_exact(4)
                .map(|chunk| {
                    let bytes =
                        <[u8; 4]>::try_from(chunk).map_err(|_| Error::Model(format!("MediaPipe tensor {} has incomplete F32 value", tensor.name)))?;
                    let value = f32::from_le_bytes(bytes);
                    if !value.is_finite() {
                        return Err(Error::Model(format!("MediaPipe tensor {} contains non-finite F32", tensor.name)));
                    }
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ValueData::Float(Arc::from(values.into_boxed_slice())))
        }
        StorageType::Float16 => {
            let values = raw
                .chunks_exact(2)
                .map(|chunk| {
                    let bytes =
                        <[u8; 2]>::try_from(chunk).map_err(|_| Error::Model(format!("MediaPipe tensor {} has incomplete F16 value", tensor.name)))?;
                    let value = half_to_f32(u16::from_le_bytes(bytes))?;
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ValueData::Float(Arc::from(values.into_boxed_slice())))
        }
        StorageType::Int32 => {
            let values = raw
                .chunks_exact(4)
                .map(|chunk| {
                    let bytes =
                        <[u8; 4]>::try_from(chunk).map_err(|_| Error::Model(format!("MediaPipe tensor {} has incomplete I32 value", tensor.name)))?;
                    Ok(i32::from_le_bytes(bytes))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ValueData::Int(Arc::from(values.into_boxed_slice())))
        }
    }
}

/// Convert an IEEE-754 binary16 bit pattern without depending on an additional crate.
fn half_to_f32(bits: u16) -> Result<f32> {
    let sign = (u32::from(bits & 0x8000)) << 16;
    let exponent = (bits >> 10) & 0x1f;
    let fraction = u32::from(bits & 0x03ff);
    let value_bits = match exponent {
        0 => {
            if fraction == 0 {
                sign
            } else {
                // Normalize a binary16 subnormal into a binary32 normal/subnormal.
                let mut fraction = fraction;
                let mut exponent32 = 113u32;
                while fraction & 0x0400 == 0 {
                    fraction <<= 1;
                    exponent32 = exponent32.saturating_sub(1);
                }
                fraction &= 0x03ff;
                sign | (exponent32 << 23) | (fraction << 13)
            }
        }
        0x1f => {
            // Preserve IEEE representation long enough to reject infinities/NaNs explicitly.
            sign | 0x7f80_0000 | (fraction << 13)
        }
        exponent => sign | ((u32::from(exponent) + 112) << 23) | (fraction << 13),
    };
    let value = f32::from_bits(value_bits);
    if value.is_finite() { Ok(value) } else { Err(Error::Model("MediaPipe Float16 constant is non-finite".into())) }
}

fn verify_artifact_bytes(bytes: &[u8]) -> Result<()> {
    let size = u64::try_from(BUNDLE_BYTES).map_err(|_| Error::Model("MediaPipe artifact byte limit overflow".into()))?;
    let spec = lightcraft_fetch::FileSpec { name: "face_landmarker-float16-v1.task", size: Some(size), sha256: Some(BUNDLE_SHA256), max: size };
    let verified = lightcraft_fetch::verify_bytes(&spec, bytes).map_err(Error::Model)?;
    if !verified {
        return Err(Error::Model(format!("MediaPipe artifact must be exactly {BUNDLE_BYTES} bytes with SHA-256 {BUNDLE_SHA256}")));
    }
    Ok(())
}

fn validate_inventory(bytes: &[u8]) -> Result<()> {
    if ENTRIES.len() != 4 {
        return Err(Error::Model(format!("MediaPipe inventory has {} entries; expected 4", ENTRIES.len())));
    }
    for entry in ENTRIES {
        let end = entry.offset.checked_add(entry.byte_len).ok_or_else(|| Error::Model(format!("MediaPipe entry {} range overflow", entry.name)))?;
        if entry.byte_len == 0 || entry.byte_len > MAX_PAYLOAD || end > bytes.len() {
            return Err(Error::Model(format!("MediaPipe entry {} has invalid exact range", entry.name)));
        }
        let raw = bytes.get(entry.offset..end).ok_or_else(|| Error::Model(format!("MediaPipe entry {} range is outside artifact", entry.name)))?;
        let size = u64::try_from(entry.byte_len).map_err(|_| Error::Model(format!("MediaPipe entry {} size overflow", entry.name)))?;
        let spec = lightcraft_fetch::FileSpec { name: entry.name, size: Some(size), sha256: Some(entry.sha256), max: size };
        if lightcraft_fetch::sha256_bytes(raw) != entry.sha256 || !lightcraft_fetch::verify_bytes(&spec, raw).map_err(Error::Model)? {
            return Err(Error::Model(format!("MediaPipe entry {} SHA-256 does not match inventory", entry.name)));
        }
    }
    for (index, first) in ENTRIES.iter().enumerate() {
        let first_end = first.offset.checked_add(first.byte_len).ok_or_else(|| Error::Model("MediaPipe entry range overflow".into()))?;
        for second in ENTRIES.iter().skip(index + 1) {
            let second_end = second.offset.checked_add(second.byte_len).ok_or_else(|| Error::Model("MediaPipe entry range overflow".into()))?;
            if first.offset < second_end && second.offset < first_end {
                let identical = first.offset == second.offset && first.byte_len == second.byte_len && first.sha256 == second.sha256;
                if !identical {
                    return Err(Error::Model(format!("MediaPipe entries {} and {} partially overlap", first.name, second.name)));
                }
            }
        }
    }
    Ok(())
}

fn validate_graphs(bytes: &[u8]) -> Result<()> {
    if GRAPHS.len() != GRAPH_COUNT {
        return Err(Error::Model(format!("MediaPipe inventory has {} graphs; expected {GRAPH_COUNT}", GRAPHS.len())));
    }
    let mut ids = HashSet::with_capacity(GRAPHS.len());
    let mut total_tensors = 0usize;
    let mut total_nodes = 0usize;
    let mut all_ranges: Vec<(usize, usize, StorageType, &str)> = Vec::new();
    for graph in GRAPHS {
        if graph.name.is_empty() || !ids.insert(graph.id) {
            return Err(Error::Model("MediaPipe inventory has duplicate or empty graph identity".into()));
        }
        if graph.tensors.is_empty() || graph.tensors.len() > MAX_TENSORS || graph.nodes.len() > MAX_NODES {
            return Err(Error::Model(format!("MediaPipe graph {} exceeds descriptor bounds", graph.name)));
        }
        total_tensors = total_tensors.checked_add(graph.tensors.len()).ok_or_else(|| Error::Model("MediaPipe tensor count overflow".into()))?;
        total_nodes = total_nodes.checked_add(graph.nodes.len()).ok_or_else(|| Error::Model("MediaPipe node count overflow".into()))?;
        if total_tensors > MAX_TOTAL_TENSORS || total_nodes > MAX_TOTAL_NODES {
            return Err(Error::Model("MediaPipe descriptor count exceeds global bounds".into()));
        }
        validate_graph_tensors(graph, bytes)?;
        all_ranges.extend(graph.tensors.iter().filter_map(|tensor| tensor.offset.map(|offset| (offset, tensor.byte_len, tensor.dtype, tensor.name))));
        validate_indices(graph)?;
        validate_nodes(graph)?;
    }
    validate_nonoverlapping_ranges(&all_ranges)?;
    Ok(())
}

fn validate_graph_tensors(graph: &GraphSpec, bytes: &[u8]) -> Result<()> {
    let mut ranges: Vec<(usize, usize, StorageType, &str)> = Vec::new();
    let mut names = HashSet::with_capacity(graph.tensors.len());
    for tensor in graph.tensors {
        if tensor.name.is_empty()
            || !names.insert(tensor.name)
            || tensor.shape.len() > MAX_RANK
            || tensor.shape.iter().any(|dimension| *dimension == 0)
        {
            return Err(Error::Model(format!("MediaPipe graph {} has malformed tensor {}", graph.name, tensor.name)));
        }
        let elements = tensor
            .shape
            .iter()
            .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
            .ok_or_else(|| Error::Model(format!("MediaPipe tensor {} shape overflows", tensor.name)))?;
        if elements > MAX_ELEMENTS {
            return Err(Error::Model(format!("MediaPipe tensor {} exceeds element bound", tensor.name)));
        }
        let width = match tensor.dtype {
            StorageType::Float16 => 2,
            StorageType::Float32 | StorageType::Int32 => 4,
        };
        if let Some(offset) = tensor.offset {
            let expected =
                elements.checked_mul(width).ok_or_else(|| Error::Model(format!("MediaPipe tensor {} byte length overflows", tensor.name)))?;
            if tensor.byte_len != expected || tensor.byte_len > MAX_PAYLOAD || tensor.byte_len % width != 0 || offset % width != 0 {
                return Err(Error::Model(format!("MediaPipe tensor {} byte length does not match shape", tensor.name)));
            }
            let end = offset.checked_add(tensor.byte_len).ok_or_else(|| Error::Model(format!("MediaPipe tensor {} range overflows", tensor.name)))?;
            if end > bytes.len() {
                return Err(Error::Model(format!("MediaPipe tensor {} range is outside artifact", tensor.name)));
            }
            ranges.push((offset, tensor.byte_len, tensor.dtype, tensor.name));
        } else if tensor.byte_len != 0 {
            return Err(Error::Model(format!("MediaPipe runtime tensor {} has constant payload bytes", tensor.name)));
        }
    }
    validate_nonoverlapping_ranges(&ranges)?;
    Ok(())
}

fn validate_nonoverlapping_ranges(ranges: &[(usize, usize, StorageType, &str)]) -> Result<()> {
    for (index, first) in ranges.iter().enumerate() {
        let first_end = first.0.checked_add(first.1).ok_or_else(|| Error::Model("MediaPipe tensor range overflow".into()))?;
        for second in ranges.iter().skip(index + 1) {
            let second_end = second.0.checked_add(second.1).ok_or_else(|| Error::Model("MediaPipe tensor range overflow".into()))?;
            if first.0 < second_end && second.0 < first_end {
                let identical = first.0 == second.0 && first.1 == second.1 && first.2 == second.2;
                if !identical {
                    return Err(Error::Model(format!("MediaPipe tensor ranges {} and {} partially overlap", first.3, second.3)));
                }
            }
        }
    }
    Ok(())
}

fn validate_indices(graph: &GraphSpec) -> Result<()> {
    let mut graph_inputs = HashSet::with_capacity(graph.inputs.len());
    let mut graph_outputs = HashSet::with_capacity(graph.outputs.len());
    for (kind, indices) in [("input", graph.inputs), ("output", graph.outputs)] {
        if indices.is_empty() {
            return Err(Error::Model(format!("MediaPipe graph {} has no {kind} tensors", graph.name)));
        }
        for index in indices {
            if *index >= graph.tensors.len() {
                return Err(Error::Model(format!("MediaPipe graph {} {kind} index {} is out of range", graph.name, index)));
            }
            let Some(tensor) = graph.tensors.get(*index) else {
                return Err(Error::Model(format!("MediaPipe graph {} {kind} index {} is out of range", graph.name, index)));
            };
            if tensor.offset.is_some() {
                return Err(Error::Model(format!("MediaPipe graph {} {kind} tensor {} is not a runtime value", graph.name, index)));
            }
            let seen = if kind == "input" { &mut graph_inputs } else { &mut graph_outputs };
            if !seen.insert(*index) {
                return Err(Error::Model(format!("MediaPipe graph {} has duplicate {kind} index {}", graph.name, index)));
            }
        }
    }
    let mut available = graph.tensors.iter().enumerate().filter_map(|(index, tensor)| tensor.offset.map(|_| index)).collect::<HashSet<_>>();
    available.extend(graph_inputs.iter().copied());
    let mut outputs = HashSet::with_capacity(graph.nodes.len());
    for node in graph.nodes {
        if node.output >= graph.tensors.len() || node.inputs.iter().any(|index| *index >= graph.tensors.len()) {
            return Err(Error::Model(format!("MediaPipe graph {} has an out-of-range node index", graph.name)));
        }
        if node.inputs.iter().any(|index| !available.contains(index)) {
            return Err(Error::Model(format!("MediaPipe graph {} has a node input without an available producer", graph.name)));
        }
        let Some(output) = graph.tensors.get(node.output) else {
            return Err(Error::Model(format!("MediaPipe graph {} node output {} is out of range", graph.name, node.output)));
        };
        if graph_inputs.contains(&node.output) || output.offset.is_some() || !outputs.insert(node.output) {
            return Err(Error::Model(format!("MediaPipe graph {} has an invalid or duplicate node output", graph.name)));
        }
        available.insert(node.output);
    }
    if graph_outputs.iter().any(|index| !available.contains(index)) {
        return Err(Error::Model(format!("MediaPipe graph {} has an output without an available producer", graph.name)));
    }
    Ok(())
}

fn validate_nodes(graph: &GraphSpec) -> Result<()> {
    for node in graph.nodes {
        let output = graph.tensors.get(node.output).ok_or_else(|| Error::Model(format!("MediaPipe graph {} node output is missing", graph.name)))?;
        let inputs = node.inputs.iter().filter_map(|index| graph.tensors.get(*index)).collect::<Vec<_>>();
        if inputs.iter().any(|tensor| tensor.dtype == StorageType::Int32) && !allows_integer_inputs(node.op) {
            return Err(Error::Model(format!("MediaPipe graph {} uses integer control in {:?}", graph.name, node.op)));
        }
        if output.dtype == StorageType::Int32 {
            return Err(Error::Model(format!("MediaPipe graph {} produces integer output from {:?}", graph.name, node.op)));
        }
        validate_op_shape(node, inputs.as_slice(), output, graph.name)?;
    }
    Ok(())
}

fn allows_integer_inputs(op: Op) -> bool {
    matches!(op, Op::Pad | Op::Reshape { .. } | Op::Transpose | Op::StridedSlice { .. } | Op::Mean { .. } | Op::Sum { .. })
}

fn float_input<'a>(inputs: &'a [&'a TensorSpec], index: usize, graph_name: &str) -> Result<&'a TensorSpec> {
    let tensor = inputs.get(index).copied().ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} node has too few inputs")))?;
    if tensor.dtype == StorageType::Int32 {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} node expects float input")));
    }
    Ok(tensor)
}

fn same_shape(a: &TensorSpec, b: &TensorSpec, graph_name: &str) -> Result<()> {
    if a.shape != b.shape {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} node shape mismatch")));
    }
    Ok(())
}

fn broadcast_shape(a: &[usize], b: &[usize]) -> Option<Vec<usize>> {
    let rank = a.len().max(b.len());
    let mut shape = vec![1usize; rank];
    for index in 0..rank {
        let reverse_index = index.checked_add(1)?;
        let a_dimension = a.len().checked_sub(reverse_index).and_then(|index| a.get(index)).copied().unwrap_or(1);
        let b_dimension = b.len().checked_sub(reverse_index).and_then(|index| b.get(index)).copied().unwrap_or(1);
        if a_dimension != b_dimension && a_dimension != 1 && b_dimension != 1 {
            return None;
        }
        let output_index = rank.checked_sub(index + 1)?;
        let slot = shape.get_mut(output_index)?;
        *slot = a_dimension.max(b_dimension);
    }
    Some(shape)
}

fn validate_op_shape(node: &Node, inputs: &[&TensorSpec], output: &TensorSpec, graph_name: &str) -> Result<()> {
    match node.op {
        Op::Add { .. } | Op::Sub { .. } | Op::Mul { .. } | Op::Div { .. } | Op::SquaredDifference => {
            let first = float_input(inputs, 0, graph_name)?;
            let second = float_input(inputs, 1, graph_name)?;
            if broadcast_shape(first.shape, second.shape).as_deref() != Some(output.shape) {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} broadcast shape mismatch")));
            }
        }
        Op::Relu | Op::Logistic | Op::Neg | Op::Sqrt | Op::Rsqrt => {
            let input = float_input(inputs, 0, graph_name)?;
            same_shape(input, output, graph_name)?;
        }
        Op::Prelu => {
            let input = float_input(inputs, 0, graph_name)?;
            let alpha = float_input(inputs, 1, graph_name)?;
            same_shape(input, output, graph_name)?;
            if alpha.shape.len() > input.shape.len() {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} PReLU alpha rank is too large")));
            }
        }
        Op::Dequantize => {
            let input = inputs.first().copied().ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} dequantize has no input")))?;
            if input.dtype == StorageType::Int32 || output.dtype == StorageType::Int32 || input.shape != output.shape {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} has invalid dequantize tensors")));
            }
        }
        Op::Pad => {
            let input = float_input(inputs, 0, graph_name)?;
            if input.shape.len() != output.shape.len() {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} pad rank mismatch")));
            }
        }
        Op::Reshape { new_shape } => {
            let input = float_input(inputs, 0, graph_name)?;
            if new_shape.len() > MAX_RANK
                || new_shape.iter().filter(|dimension| **dimension == -1).count() > 1
                || new_shape.iter().any(|dimension| *dimension == 0 || *dimension < -1)
            {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} has invalid reshape descriptor")));
            }
            if new_shape.is_empty() {
                let control =
                    inputs.get(1).copied().ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} dynamic reshape has no shape input")))?;
                if control.dtype != StorageType::Int32 {
                    return Err(Error::Model(format!("MediaPipe graph {graph_name} dynamic reshape shape input is not Int32")));
                }
            }
            let mut known = 1usize;
            for dimension in new_shape.iter().copied().filter(|dimension| *dimension > 0) {
                let dimension = usize::try_from(dimension).map_err(|_| Error::Model("MediaPipe reshape dimension overflow".into()))?;
                known = known.checked_mul(dimension).ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} reshape overflows")))?;
            }
            let input_elements = input
                .shape
                .iter()
                .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
                .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} reshape input overflows")))?;
            if !new_shape.is_empty() && new_shape.iter().all(|dimension| *dimension != -1) && known != input_elements
                || output.shape.iter().try_fold(1usize, |count, dimension| count.checked_mul(*dimension)) != Some(input_elements)
            {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} reshape element count mismatch")));
            }
        }
        Op::Concat { axis, .. } => {
            let first = float_input(inputs, 0, graph_name)?;
            let rank = first.shape.len();
            let axis = normalize_axis(axis, rank, graph_name)?;
            for input in inputs.iter().skip(1) {
                if input.dtype == StorageType::Int32 {
                    return Err(Error::Model(format!("MediaPipe graph {graph_name} concat expects float inputs")));
                }
                let input = *input;
                if input.shape.len() != rank
                    || input.shape.iter().enumerate().any(|(index, dimension)| index != axis && *dimension != first.shape[index])
                {
                    return Err(Error::Model(format!("MediaPipe graph {graph_name} concat shape mismatch")));
                }
            }
            let expected = inputs
                .iter()
                .try_fold(0usize, |sum, input| sum.checked_add(input.shape[axis]))
                .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} concat dimension overflows")))?;
            if output.shape.len() != rank || output.shape[axis] != expected {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} concat output mismatch")));
            }
        }
        Op::Transpose => {
            let input = float_input(inputs, 0, graph_name)?;
            if input.shape.len() != output.shape.len() || element_count(input.shape) != element_count(output.shape) {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} transpose shape mismatch")));
            }
        }
        Op::StridedSlice { .. } => {
            let input = float_input(inputs, 0, graph_name)?;
            if output.shape.len() > input.shape.len() {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} strided slice increased rank")));
            }
        }
        Op::Mean { keep_dims } => {
            let input = float_input(inputs, 0, graph_name)?;
            if keep_dims {
                if input.shape.len() != output.shape.len() {
                    return Err(Error::Model(format!("MediaPipe graph {graph_name} mean rank mismatch")));
                }
            } else if output.shape.len() >= input.shape.len() {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} mean did not reduce rank")));
            }
        }
        Op::Sum { keep_dims } => {
            let input = float_input(inputs, 0, graph_name)?;
            if keep_dims {
                if input.shape.len() != output.shape.len() {
                    return Err(Error::Model(format!("MediaPipe graph {graph_name} sum rank mismatch")));
                }
            } else if output.shape.len() >= input.shape.len() {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} sum did not reduce rank")));
            }
        }
        Op::Conv2d { stride_h, stride_w, dilation_h, dilation_w, padding, .. } => {
            validate_conv(inputs, output, stride_h, stride_w, dilation_h, dilation_w, padding, graph_name, false, 1)?;
        }
        Op::DepthwiseConv2d { stride_h, stride_w, dilation_h, dilation_w, depth_multiplier, padding, .. } => {
            if depth_multiplier == 0 {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} has zero depth multiplier")));
            }
            validate_conv(inputs, output, stride_h, stride_w, dilation_h, dilation_w, padding, graph_name, true, depth_multiplier)?;
        }
        Op::MaxPool2d { stride_h, stride_w, filter_h, filter_w, padding, .. } => {
            if stride_h == 0 || stride_w == 0 || filter_h == 0 || filter_w == 0 {
                return Err(Error::Model(format!("MediaPipe graph {graph_name} has zero pooling parameter")));
            }
            let input = float_input(inputs, 0, graph_name)?;
            validate_pool(input, output, stride_h, stride_w, filter_h, filter_w, padding, graph_name)?;
        }
    }
    Ok(())
}

fn validate_conv(
    inputs: &[&TensorSpec],
    output: &TensorSpec,
    stride_h: usize,
    stride_w: usize,
    dilation_h: usize,
    dilation_w: usize,
    padding: Padding,
    graph_name: &str,
    depthwise: bool,
    depth_multiplier: usize,
) -> Result<()> {
    if stride_h == 0 || stride_w == 0 || dilation_h == 0 || dilation_w == 0 || stride_h != stride_w || dilation_h != dilation_w {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} has unsupported asymmetric convolution parameters")));
    }
    let input = float_input(inputs, 0, graph_name)?;
    let filter = float_input(inputs, 1, graph_name)?;
    let bias = float_input(inputs, 2, graph_name)?;
    if inputs.len() != 3 || input.shape.len() != 4 || filter.shape.len() != 4 || bias.shape.len() != 1 || output.shape.len() != 4 {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} convolution descriptor has invalid rank or input count")));
    }
    let input_channels =
        *input.shape.get(3).ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} convolution input has no channels")))?;
    let output_channels = if depthwise {
        if filter.shape.first().copied() != Some(1) {
            return Err(Error::Model(format!("MediaPipe graph {graph_name} depthwise filter multiplier dimension is not one")));
        }
        input_channels
            .checked_mul(depth_multiplier)
            .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} depthwise channel count overflows")))?
    } else {
        if filter.shape.get(3).copied() != Some(input_channels) {
            return Err(Error::Model(format!("MediaPipe graph {graph_name} convolution filter Cin does not match input channels")));
        }
        *filter.shape.first().ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} convolution filter has no Cout")))?
    };
    if depthwise {
        let expected_filter_channels = input_channels
            .checked_mul(depth_multiplier)
            .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} depthwise channel count overflows")))?;
        if filter.shape.get(3).copied() != Some(expected_filter_channels) {
            return Err(Error::Model(format!("MediaPipe graph {graph_name} depthwise filter channels do not match multiplier")));
        }
    }
    if bias.shape.first().copied() != Some(output_channels) {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} convolution bias does not match Cout")));
    }
    let filter_h = *filter.shape.get(1).ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} convolution filter has no height")))?;
    let filter_w = *filter.shape.get(2).ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} convolution filter has no width")))?;
    validate_spatial_output(input, output, filter_h, filter_w, stride_h, dilation_h, padding, output_channels, graph_name)?;
    Ok(())
}

fn validate_pool(
    input: &TensorSpec,
    output: &TensorSpec,
    stride_h: usize,
    stride_w: usize,
    filter_h: usize,
    filter_w: usize,
    padding: Padding,
    graph_name: &str,
) -> Result<()> {
    if stride_h == 0 || stride_w == 0 || filter_h == 0 || filter_w == 0 || stride_h != stride_w {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} has unsupported asymmetric pooling parameters")));
    }
    if input.shape.len() != 4 || output.shape.len() != 4 {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} pooling rank is not four")));
    }
    let channels = *input.shape.get(3).ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} pooling input has no channels")))?;
    validate_spatial_output(input, output, filter_h, filter_w, stride_h, 1, padding, channels, graph_name)
}

fn validate_spatial_output(
    input: &TensorSpec,
    output: &TensorSpec,
    filter_h: usize,
    filter_w: usize,
    stride_h: usize,
    dilation: usize,
    padding: Padding,
    output_channels: usize,
    graph_name: &str,
) -> Result<()> {
    let input_batch = *input.shape.first().ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} input has no batch dimension")))?;
    let input_h = *input.shape.get(1).ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} input has no height dimension")))?;
    let input_w = *input.shape.get(2).ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} input has no width dimension")))?;
    let expected_h = spatial_output_dim(input_h, filter_h, stride_h, dilation, padding, graph_name)?;
    let expected_w = spatial_output_dim(input_w, filter_w, stride_h, dilation, padding, graph_name)?;
    if output.shape.first().copied() != Some(input_batch)
        || output.shape.get(1).copied() != Some(expected_h)
        || output.shape.get(2).copied() != Some(expected_w)
        || output.shape.get(3).copied() != Some(output_channels)
    {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} convolution or pooling output shape mismatch")));
    }
    Ok(())
}

fn spatial_output_dim(input: usize, filter: usize, stride: usize, dilation: usize, padding: Padding, graph_name: &str) -> Result<usize> {
    if filter == 0 || stride == 0 || dilation == 0 {
        return Err(Error::Model(format!("MediaPipe graph {graph_name} has zero spatial parameter")));
    }
    let effective = filter
        .checked_sub(1)
        .and_then(|value| value.checked_mul(dilation))
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} effective kernel overflows")))?;
    match padding {
        Padding::Same => input
            .checked_add(stride - 1)
            .map(|value| value / stride)
            .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} same padding output overflows"))),
        Padding::Valid => input
            .checked_sub(effective)
            .and_then(|value| (value / stride).checked_add(1))
            .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} valid kernel exceeds input"))),
    }
}

fn normalize_axis(axis: i32, rank: usize, graph_name: &str) -> Result<usize> {
    let rank_i32 = i32::try_from(rank).map_err(|_| Error::Model(format!("MediaPipe graph {graph_name} rank overflow")))?;
    let axis = if axis < 0 { axis.checked_add(rank_i32) } else { Some(axis) }
        .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} concat axis overflow")))?;
    usize::try_from(axis)
        .ok()
        .filter(|axis| *axis < rank)
        .ok_or_else(|| Error::Model(format!("MediaPipe graph {graph_name} concat axis is out of range")))
}

fn element_count(shape: &[usize]) -> Option<usize> {
    shape.iter().try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_conversion_handles_zero_subnormal_and_normal() {
        assert_eq!(half_to_f32(0).ok(), Some(0.0));
        assert_eq!(half_to_f32(0x8000).ok(), Some(-0.0));
        assert_eq!(half_to_f32(0x0001).ok(), Some(2f32.powi(-24)));
        assert_eq!(half_to_f32(0x3c00).ok(), Some(1.0));
    }

    #[test]
    fn half_conversion_rejects_infinity_and_nan() {
        assert!(half_to_f32(0x7c00).is_err());
        assert!(half_to_f32(0x7e00).is_err());
    }

    #[test]
    fn wrong_identity_is_rejected_without_inventory_work() {
        assert!(verify_artifact_bytes(&[]).is_err());
    }

    #[test]
    fn scalar_int_controls_remain_host_values() {
        const SPEC: TensorSpec = TensorSpec { name: "scalar", shape: &[], dtype: StorageType::Int32, offset: Some(0), byte_len: 4 };
        let value = decode_value(&SPEC, &7i32.to_le_bytes()).ok();
        assert!(matches!(value, Some(ValueData::Int(values)) if values.as_ref().len() == 1 && values.as_ref().first() == Some(&7)));
    }

    #[test]
    fn malformed_tensor_descriptor_is_rejected_before_decode() {
        const SPEC: TensorSpec = TensorSpec { name: "bad", shape: &[2], dtype: StorageType::Float32, offset: Some(0), byte_len: 4 };
        const TENSORS: &[TensorSpec] = &[SPEC];
        const INPUTS: &[usize] = &[0];
        const OUTPUTS: &[usize] = &[0];
        const NODES: &[Node] = &[];
        const GRAPH: GraphSpec =
            GraphSpec { id: GraphId::Detector, name: "malformed", inputs: INPUTS, outputs: OUTPUTS, tensors: TENSORS, nodes: NODES };
        assert!(validate_graph_tensors(&GRAPH, &[0; 4]).is_err());
    }

    #[test]
    fn broadcast_and_dynamic_reshape_descriptors_are_accepted() {
        assert_eq!(broadcast_shape(&[1, 4, 5, 6], &[]), Some(vec![1, 4, 5, 6]));
        assert_eq!(broadcast_shape(&[1, 4, 5, 6], &[6]), Some(vec![1, 4, 5, 6]));
        assert_eq!(broadcast_shape(&[2, 1], &[1, 3]), Some(vec![2, 3]));
        assert!(broadcast_shape(&[2], &[3]).is_none());

        const DATA: TensorSpec = TensorSpec { name: "data", shape: &[1, 2, 3], dtype: StorageType::Float32, offset: None, byte_len: 0 };
        const SHAPE: TensorSpec = TensorSpec { name: "shape", shape: &[3], dtype: StorageType::Int32, offset: Some(0), byte_len: 12 };
        const OUTPUT: TensorSpec = TensorSpec { name: "output", shape: &[1, 2, 3], dtype: StorageType::Float32, offset: None, byte_len: 0 };
        const NODE: Node = Node { op: Op::Reshape { new_shape: &[] }, inputs: &[0, 1], output: 2 };
        let inputs = [&DATA, &SHAPE];
        assert!(validate_op_shape(&NODE, &inputs, &OUTPUT, "test").is_ok());
    }
}
