//! Experimental raw graph execution for exact MediaPipe Face Landmarker v1 bytes.
//!
//! Callers supply preprocessed graph tensors. [`crate::mediapipe_photo`] composes
//! experimental RGB preprocessing, detector decoding, crops & landmark mapping.
//! Eye-state calibration remains separate. No output authorizes a culling decision.
//! Source implementation only: reference parity, accuracy & latency are unqualified.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{DType, Device, Tensor};

use crate::mediapipe_artifact::Bundle;
use crate::mediapipe_graph::{GraphId, LoadedGraph, Node, Op, StorageType, TensorSpec, Value};
use crate::{Error, Result};

const MAX_ELEMENTS: usize = 16_000_000;
const MAX_LIVE_ELEMENTS: usize = 64_000_000;
const MAX_NODE_INPUTS: usize = 64;

/// Raw detector heads. Anchor decoding & score calibration have not been applied.
pub struct DetectorOutput {
    /// `[1,896,16]` regressors in the source graph's output order.
    pub regressors: Tensor,
    /// `[1,896,1]` classifier logits, before sigmoid/thresholding.
    pub classifier_logits: Tensor,
}

/// Raw FaceMesh graph outputs, without crop-coordinate remapping.
pub struct LandmarksOutput {
    /// `[1,1,1,1434]`: 478 raw xyz coordinate triples.
    pub coordinates: Tensor,
    /// `[1,1,1,1]` source output `Identity_1`; no confidence calibration.
    pub auxiliary_1: Tensor,
    /// `[1,1]` source output `Identity_2`; no confidence calibration.
    pub auxiliary_2: Tensor,
}

impl Bundle {
    /// Execute detector on finite F32 NHWC `[1,128,128,3]` input.
    /// Source preprocessing/normalization remains caller's qualification obligation.
    pub fn forward_detector(&self, input: &Tensor) -> Result<DetectorOutput> {
        let values = execute_graph(self.graph(GraphId::Detector)?, input)?;
        Ok(DetectorOutput { regressors: output(&values, 0)?, classifier_logits: output(&values, 1)? })
    }

    /// Execute FaceMesh on finite F32 NHWC `[1,256,256,3]` input.
    /// This API does not choose, rotate, normalize or enlarge a face crop.
    pub fn forward_landmarks(&self, input: &Tensor) -> Result<LandmarksOutput> {
        let values = execute_graph(self.graph(GraphId::Landmarks)?, input)?;
        Ok(LandmarksOutput { coordinates: output(&values, 0)?, auxiliary_1: output(&values, 1)?, auxiliary_2: output(&values, 2)? })
    }

    /// Execute blendshape graph on finite F32 `[1,146,2]` source-ordered landmarks.
    /// Returns raw `[52]` coefficients, not calibrated eye-open probabilities.
    pub fn forward_blendshapes(&self, input: &Tensor) -> Result<Tensor> {
        let values = execute_graph(self.graph(GraphId::Blendshapes)?, input)?;
        output(&values, 0)
    }
}

fn output(values: &[Tensor], index: usize) -> Result<Tensor> {
    values.get(index).cloned().ok_or_else(|| Error::Model("MediaPipe raw graph output is missing".into()))
}

fn execute_graph(graph: &LoadedGraph, input: &Tensor) -> Result<Vec<Tensor>> {
    let spec = graph.spec;
    if spec.inputs.len() != 1 || graph.constants.len() != spec.tensors.len() {
        return Err(Error::Model("MediaPipe loaded graph has invalid input/constant layout".into()));
    }
    let input_index = *spec.inputs.first().ok_or_else(|| Error::Model("MediaPipe graph input is missing".into()))?;
    let input_spec = spec.tensors.get(input_index).ok_or_else(|| Error::Model("MediaPipe graph input index is invalid".into()))?;
    let input_value = Value::Float(input.clone());
    validate_value(&input_value, input_spec)?;
    check_live_budget(
        &graph.constants,
        elements(input_spec.shape)?.checked_mul(4).ok_or_else(|| Error::Model("MediaPipe input budget overflow".into()))?,
    )?;
    finite(input)?;
    for constant in graph.constants.iter().flatten() {
        if let Value::Float(tensor) = constant
            && !tensor.device().same_device(input.device())
        {
            return Err(Error::Model("MediaPipe input & loaded constants must share device".into()));
        }
    }

    let mut values = graph.constants.clone();
    let input_slot = values.get_mut(input_index).ok_or_else(|| Error::Model("MediaPipe graph input slot is invalid".into()))?;
    if input_slot.is_some() {
        return Err(Error::Model("MediaPipe graph input aliases a constant".into()));
    }
    *input_slot = Some(input_value);
    let (mut uses, retain) = use_counts(graph)?;
    check_live_budget(&values, 0)?;

    for (node_index, node) in spec.nodes.iter().enumerate() {
        if node.inputs.len() > MAX_NODE_INPUTS {
            return Err(Error::Model("MediaPipe node exceeds input-count bound".into()));
        }
        let expected = spec.tensors.get(node.output).ok_or_else(|| Error::Model("MediaPipe node output metadata is invalid".into()))?;
        check_live_budget(&values, scratch_reservation(graph, node, input.device())?)?;
        let inputs = node
            .inputs
            .iter()
            .map(|index| {
                values
                    .get(*index)
                    .and_then(Option::as_ref)
                    .cloned()
                    .ok_or_else(|| Error::Model(format!("MediaPipe {} node {node_index} uses unavailable tensor {index}", spec.name)))
            })
            .collect::<Result<Vec<_>>>()?;
        let value = crate::mediapipe_ops::execute(&node.op, &inputs, expected)
            .map_err(|error| Error::Model(format!("MediaPipe {} node {node_index} {:?}: {error}", spec.name, node.op)))?;
        validate_value(&value, expected)?;
        // This offline prototype prioritizes diagnostic correctness. Per-node host
        // finite checks synchronize GPU execution; throughput remains unqualified.
        finite(value.float()?)?;
        let slot = values.get_mut(node.output).ok_or_else(|| Error::Model("MediaPipe node output slot is invalid".into()))?;
        if slot.is_some() {
            return Err(Error::Model("MediaPipe node overwrites an existing tensor".into()));
        }
        *slot = Some(value);

        // Tensor clones share storage. Release graph-local references when no later
        // node needs them; immutable bundle constants remain cached across executions.
        for index in node.inputs {
            let count = uses.get_mut(*index).ok_or_else(|| Error::Model("MediaPipe tensor use index is invalid".into()))?;
            *count = count.checked_sub(1).ok_or_else(|| Error::Model("MediaPipe tensor use count underflow".into()))?;
            if *count == 0
                && !retain.get(*index).copied().unwrap_or(false)
                && let Some(slot) = values.get_mut(*index)
            {
                *slot = None;
            }
        }
        if uses.get(node.output).copied() == Some(0)
            && !retain.get(node.output).copied().unwrap_or(false)
            && let Some(slot) = values.get_mut(node.output)
        {
            *slot = None;
        }
    }

    spec.outputs
        .iter()
        .map(|index| {
            let value = values.get(*index).and_then(Option::as_ref).ok_or_else(|| Error::Model("MediaPipe graph output is unavailable".into()))?;
            let tensor = value.float()?;
            finite(tensor)?;
            Ok(tensor.clone())
        })
        .collect()
}

fn use_counts(graph: &LoadedGraph) -> Result<(Vec<usize>, Vec<bool>)> {
    let mut uses = vec![0usize; graph.spec.tensors.len()];
    // Bundle constants stay allocated across runs; keep their slots in live-budget accounting.
    let mut retain = graph.constants.iter().map(Option::is_some).collect::<Vec<_>>();
    if retain.len() != graph.spec.tensors.len() {
        return Err(Error::Model("MediaPipe constant-retention layout is invalid".into()));
    }
    for node in graph.spec.nodes {
        for index in node.inputs {
            let count = uses.get_mut(*index).ok_or_else(|| Error::Model("MediaPipe use-count index is invalid".into()))?;
            *count = count.checked_add(1).ok_or_else(|| Error::Model("MediaPipe use count overflow".into()))?;
        }
    }
    for index in graph.spec.outputs {
        *retain.get_mut(*index).ok_or_else(|| Error::Model("MediaPipe output-retention index is invalid".into()))? = true;
    }
    Ok((uses, retain))
}

fn elements(shape: &[usize]) -> Result<usize> {
    if shape.len() > 8 || shape.contains(&0) {
        return Err(Error::Model("MediaPipe tensor rank/dimensions are invalid".into()));
    }
    shape
        .iter()
        .try_fold(1usize, |count, dim| count.checked_mul(*dim))
        .filter(|count| *count <= MAX_ELEMENTS)
        .ok_or_else(|| Error::Model("MediaPipe tensor exceeds element bound".into()))
}

/// Conservative reservation for explicit operator tensors, finite-check readbacks &
/// Candle's documented convolution workspace. Driver caches are outside this budget.
fn scratch_reservation(graph: &LoadedGraph, node: &Node, device: &Device) -> Result<usize> {
    let output = graph.spec.tensors.get(node.output).ok_or_else(|| Error::Model("MediaPipe scratch output is invalid".into()))?;
    let mut count = elements(output.shape)?;
    for index in node.inputs {
        let input = graph.spec.tensors.get(*index).ok_or_else(|| Error::Model("MediaPipe scratch input is invalid".into()))?;
        count = count.checked_add(elements(input.shape)?).ok_or_else(|| Error::Model("MediaPipe scratch size overflow".into()))?;
    }
    // Covers out-of-place arithmetic, layout copies & up to three F32 readback copies.
    let mut scratch = count.checked_mul(8).ok_or_else(|| Error::Model("MediaPipe scratch reservation overflow".into()))?;
    if matches!(node.op, Op::Conv2d { .. } | Op::DepthwiseConv2d { .. }) {
        let input = node
            .inputs
            .first()
            .and_then(|index| graph.spec.tensors.get(*index))
            .ok_or_else(|| Error::Model("MediaPipe convolution scratch input is missing".into()))?;
        let filter = node
            .inputs
            .get(1)
            .and_then(|index| graph.spec.tensors.get(*index))
            .ok_or_else(|| Error::Model("MediaPipe convolution scratch filter is missing".into()))?;
        if input.shape.len() != 4 || filter.shape.len() != 4 || output.shape.len() != 4 {
            return Err(Error::Model("MediaPipe convolution scratch rank is invalid".into()));
        }
        let channels = *input.shape.get(3).ok_or_else(|| Error::Model("MediaPipe convolution channels are missing".into()))?;
        let groups = if matches!(node.op, Op::DepthwiseConv2d { .. }) { channels } else { 1 };
        let kh = *filter.shape.get(1).ok_or_else(|| Error::Model("MediaPipe convolution kernel height is missing".into()))?;
        let kw = *filter.shape.get(2).ok_or_else(|| Error::Model("MediaPipe convolution kernel width is missing".into()))?;
        let workspace = if device.is_cpu() {
            channels.checked_div(groups).and_then(|cin| cin.checked_mul(kh)).and_then(|n| n.checked_mul(kw)).and_then(|n| n.checked_mul(512))
        } else {
            output
                .shape
                .first()
                .copied()
                .and_then(|n| output.shape.get(1).and_then(|h| n.checked_mul(*h)))
                .and_then(|n| output.shape.get(2).and_then(|w| n.checked_mul(*w)))
                .and_then(|n| n.checked_mul(channels))
                .and_then(|n| n.checked_mul(kh))
                .and_then(|n| n.checked_mul(kw))
        }
        .ok_or_else(|| Error::Model("MediaPipe convolution workspace overflow".into()))?;
        scratch = scratch.checked_add(workspace).ok_or_else(|| Error::Model("MediaPipe convolution reservation overflow".into()))?;
    }
    Ok(scratch)
}

fn validate_value(value: &Value, spec: &TensorSpec) -> Result<()> {
    let count = elements(spec.shape)?;
    match value {
        Value::Float(tensor) => {
            if spec.dtype == StorageType::Int32 || tensor.dtype() != DType::F32 || tensor.dims() != spec.shape {
                return Err(Error::Model(format!("MediaPipe tensor {} has wrong dtype/shape", spec.name)));
            }
        }
        Value::Int { values, shape } => {
            if spec.dtype != StorageType::Int32 || shape.as_slice() != spec.shape || values.len() != count {
                return Err(Error::Model(format!("MediaPipe control {} has wrong dtype/shape", spec.name)));
            }
        }
    }
    Ok(())
}

fn check_live_budget(values: &[Option<Value>], extra: usize) -> Result<()> {
    let mut count = extra;
    for value in values.iter().flatten() {
        let current = match value {
            Value::Float(tensor) => elements(tensor.dims())?,
            Value::Int { values, .. } => values.len(),
        };
        count = count.checked_add(current).ok_or_else(|| Error::Model("MediaPipe live tensor count overflow".into()))?;
        if count > MAX_LIVE_ELEMENTS {
            return Err(Error::Model("MediaPipe live tensor element budget exceeded".into()));
        }
    }
    Ok(())
}

fn finite(tensor: &Tensor) -> Result<()> {
    let _ = elements(tensor.dims())?;
    if tensor.flatten_all()?.to_vec1::<f32>()?.iter().any(|value| !value.is_finite()) {
        return Err(Error::Model("MediaPipe boundary tensor contains non-finite values".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mediapipe_graph::{GraphSpec, Node, Op};
    use candle_core::Device;

    #[test]
    fn repeated_inputs_are_counted_and_outputs_retained() {
        static TENSORS: [TensorSpec; 2] = [
            TensorSpec { name: "x", shape: &[1], dtype: StorageType::Float32, offset: None, byte_len: 0 },
            TensorSpec { name: "y", shape: &[1], dtype: StorageType::Float32, offset: None, byte_len: 0 },
        ];
        static NODES: [Node; 1] = [Node { op: Op::SquaredDifference, inputs: &[0, 0], output: 1 }];
        static SPEC: GraphSpec =
            GraphSpec { id: GraphId::Detector, name: "synthetic", inputs: &[0], outputs: &[1], tensors: &TENSORS, nodes: &NODES };
        let graph = LoadedGraph { spec: &SPEC, constants: vec![None, None] };
        let (uses, retain) = use_counts(&graph).unwrap();
        assert_eq!(uses, vec![2, 0]);
        assert_eq!(retain, vec![false, true]);
        let input = Tensor::from_vec(vec![3f32], &[1], &Device::Cpu).unwrap();
        let outputs = execute_graph(&graph, &input).unwrap();
        assert_eq!(outputs[0].to_vec1::<f32>().unwrap(), vec![0.]);
    }

    #[test]
    fn boundary_checks_reject_wrong_dtype_shape_and_nonfinite() {
        let spec = TensorSpec { name: "x", shape: &[2], dtype: StorageType::Float32, offset: None, byte_len: 0 };
        let wrong_shape = Tensor::from_vec(vec![1f32], &[1], &Device::Cpu).unwrap();
        assert!(validate_value(&Value::Float(wrong_shape), &spec).is_err());
        let wrong_type = Value::Int { values: vec![1, 2], shape: vec![2] };
        assert!(validate_value(&wrong_type, &spec).is_err());
        let invalid = Tensor::from_vec(vec![f32::NAN, 1.], &[2], &Device::Cpu).unwrap();
        assert!(finite(&invalid).is_err());
        assert!(elements(&[usize::MAX, 2]).is_err());
        assert!(elements(&[0]).is_err());
        assert_eq!(elements(&[]).unwrap(), 1);
    }

    #[test]
    fn nonfinite_intermediate_is_rejected_before_bounded_activation() {
        static TENSORS: [TensorSpec; 3] = [
            TensorSpec { name: "x", shape: &[1], dtype: StorageType::Float32, offset: None, byte_len: 0 },
            TensorSpec { name: "overflow", shape: &[1], dtype: StorageType::Float32, offset: None, byte_len: 0 },
            TensorSpec { name: "bounded", shape: &[1], dtype: StorageType::Float32, offset: None, byte_len: 0 },
        ];
        static NODES: [Node; 2] = [
            Node { op: Op::Mul { activation: crate::mediapipe_graph::Activation::None }, inputs: &[0, 0], output: 1 },
            Node { op: Op::Logistic, inputs: &[1], output: 2 },
        ];
        static SPEC: GraphSpec =
            GraphSpec { id: GraphId::Detector, name: "synthetic-overflow", inputs: &[0], outputs: &[2], tensors: &TENSORS, nodes: &NODES };
        let graph = LoadedGraph { spec: &SPEC, constants: vec![None, None, None] };
        let input = Tensor::from_vec(vec![f32::MAX], &[1], &Device::Cpu).unwrap();
        assert!(execute_graph(&graph, &input).is_err());
    }
}
