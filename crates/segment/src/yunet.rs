//! Bounded YuNet 2023mar face detector prototype.
//!
//! This module intentionally accepts only the pinned OpenCV Zoo artifact.  It parses the small
//! ONNX protobuf surface needed by that graph, validates its fingerprint and topology, then
//! executes its eight permitted operators with Candle.  Post-processing remains explicit here:
//! preprocessing, score fusion, box/keypoint decoding and NMS are not encoded in ONNX.
//! This prototype freezes its own baseline as RGB-to-BGR 0..255 input, aspect-preserving
//! top-left resize into 640x640, grid-origin `(column, row)` decoding, square-root score fusion,
//! source-space xyxy boxes and bounded float IoU NMS.  These choices are experimental and are not
//! claimed to match OpenCV's reference post-processing.
//!
//! The model is a research prototype.  Loading it does not qualify accuracy, eye-state
//! classification or production performance.

#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;

use candle_core::{DType, Device, Tensor};

const MODEL_SIZE: u64 = 232_589;
const MODEL_SHA256: &str = "8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4";
const INPUT_SIDE: usize = 640;
const MAX_PROTO_BYTES: usize = 16 * 1024 * 1024;
const MAX_PROTO_STRING: usize = 4096;
const MAX_TENSOR_ELEMENTS: usize = 1_000_000;
const MAX_CANDIDATES: usize = 10_000;
const MAX_DETECTIONS: usize = 300;

/// YuNet's fixed tensor side.
pub const INPUT_SIZE: usize = INPUT_SIDE;
/// Pinned OpenCV Zoo artifact SHA-256.
pub const PINNED_SHA256: &str = MODEL_SHA256;

const EXPECTED_OUTPUTS: [&str; 12] =
    ["cls_8", "cls_16", "cls_32", "obj_8", "obj_16", "obj_32", "bbox_8", "bbox_16", "bbox_32", "kps_8", "kps_16", "kps_32"];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("YuNet model: {0}")]
    Model(String),
    #[error("YuNet model I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Candle(#[from] candle_core::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Decoded face candidate. `bbox` is `[x0, y0, x1, y1]` in source-image pixels. Keypoints are
/// left as five `(x, y)` points for callers that need landmarks; this prototype does not infer
/// open/closed eye state from them.
#[derive(Clone, Debug, PartialEq)]
pub struct Detection {
    pub score: f32,
    pub bbox: [f32; 4],
    pub keypoints: [[f32; 2]; 5],
}

/// Explicit input policy. RGB bytes are letterboxed with nearest-neighbour sampling into the
/// fixed NCHW tensor. YuNet's pinned OpenCV reference uses BGR bytes in 0..255; swap_rb therefore
/// defaults to true for this RGB-facing API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preprocess {
    pub mean: [f32; 3],
    pub scale: f32,
    pub swap_rb: bool,
}

impl Default for Preprocess {
    fn default() -> Self {
        Self { mean: [0.0; 3], scale: 1.0, swap_rb: true }
    }
}

/// Fixed graph outputs before YuNet decoding.
pub struct RawOutputs {
    pub cls: [Tensor; 3],
    pub obj: [Tensor; 3],
    pub bbox: [Tensor; 3],
    pub keypoints: [Tensor; 3],
}

/// Loaded, fingerprint-checked YuNet graph.
pub struct YuNet {
    nodes: Vec<Node>,
    initializers: HashMap<String, Initializer>,
    /// Device-resident FLOAT initializers; avoids per-frame host-to-device uploads.
    tensors: HashMap<String, Tensor>,
    device: Device,
    preprocess: Preprocess,
}

impl YuNet {
    /// Pinned model digest exposed for qualification receipts.
    pub const PINNED_SHA256: &'static str = MODEL_SHA256;

    /// Load exact pinned ONNX bytes, checking size and SHA-256 through lightcraft-fetch.
    pub fn load(path: &Path, device: &Device) -> Result<Self> {
        let spec = lightcraft_fetch::FileSpec {
            name: "face_detection_yunet_2023mar.onnx",
            size: Some(MODEL_SIZE),
            sha256: Some(MODEL_SHA256),
            max: MODEL_SIZE,
        };
        let mut file = std::fs::File::open(path)?;
        let mut bytes = Vec::with_capacity(MODEL_SIZE as usize + 1);
        file.by_ref().take(MODEL_SIZE.saturating_add(1)).read_to_end(&mut bytes)?;
        if !lightcraft_fetch::verify_bytes(&spec, &bytes).map_err(Error::Model)? {
            return Err(Error::Model(format!("{}: size or SHA-256 does not match pinned YuNet artifact", path.display())));
        }
        Self::from_verified_bytes(&bytes, device.clone())
    }

    fn from_verified_bytes(bytes: &[u8], device: Device) -> Result<Self> {
        if bytes.len() != MODEL_SIZE as usize || bytes.len() > MAX_PROTO_BYTES {
            return Err(Error::Model(format!("YuNet artifact has {} bytes; expected {MODEL_SIZE}", bytes.len())));
        }
        let parsed = Graph::parse(bytes)?;
        parsed.validate()?;
        let mut tensors = HashMap::with_capacity(parsed.initializers.len());
        for (name, initializer) in &parsed.initializers {
            if initializer.data_type != 1 {
                continue;
            }
            let tensor = Tensor::from_vec(initializer.data.clone(), initializer.dims.as_slice(), &device)?;
            ensure_finite(&tensor)?;
            tensors.insert(name.clone(), tensor);
        }
        Ok(Self { nodes: parsed.nodes, initializers: parsed.initializers, tensors, device, preprocess: Preprocess::default() })
    }

    /// Set explicit preprocessing policy.
    pub fn with_preprocess(mut self, preprocess: Preprocess) -> Result<Self> {
        if !preprocess.scale.is_finite() || preprocess.scale <= 0.0 || preprocess.scale > 16.0 || preprocess.mean.iter().any(|v| !v.is_finite()) {
            return Err(Error::Model("preprocessing policy contains nonfinite or out-of-range values".into()));
        }
        self.preprocess = preprocess;
        Ok(self)
    }

    /// Build fixed input tensor from interleaved RGB bytes. Empty, oversized or overflowing
    /// dimensions are rejected before any allocation or indexing.
    pub fn preprocess_rgb(&self, rgb: &[u8], width: usize, height: usize) -> Result<Tensor> {
        preprocess_rgb(rgb, width, height, self.preprocess, &self.device)
    }

    /// Execute graph and return raw sigmoid/classification, objectness, box and keypoint heads.
    pub fn forward(&self, input: &Tensor) -> Result<RawOutputs> {
        validate_input_tensor(input)?;
        let mut values = HashMap::<String, Tensor>::new();
        values.insert("input".into(), input.clone());
        let mut last_use = HashMap::<&str, usize>::new();
        for (index, node) in self.nodes.iter().enumerate() {
            for name in &node.inputs {
                last_use.insert(name.as_str(), index);
            }
        }
        for (index, node) in self.nodes.iter().enumerate() {
            let output = self.execute(node, &values)?;
            let Some(name) = node.outputs.first() else {
                return Err(Error::Model("node has no output".into()));
            };
            values.insert(name.clone(), output);
            for input_name in &node.inputs {
                if last_use.get(input_name.as_str()) == Some(&index) && input_name != "input" {
                    values.remove(input_name);
                }
            }
        }
        let mut take = |name: &str| values.remove(name).ok_or_else(|| Error::Model(format!("graph did not produce {name}")));
        let outputs = RawOutputs {
            cls: [take("cls_8")?, take("cls_16")?, take("cls_32")?],
            obj: [take("obj_8")?, take("obj_16")?, take("obj_32")?],
            bbox: [take("bbox_8")?, take("bbox_16")?, take("bbox_32")?],
            keypoints: [take("kps_8")?, take("kps_16")?, take("kps_32")?],
        };
        // Keep intermediate tensors on device; validate bounded public heads once before return.
        for tensor in outputs.cls.iter().chain(outputs.obj.iter()).chain(outputs.bbox.iter()).chain(outputs.keypoints.iter()) {
            ensure_finite(tensor)?;
        }
        Ok(outputs)
    }

    /// Preprocess, execute, decode and NMS one RGB image. Thresholds are inclusive and bounded
    /// to `[0, 1]`; output candidates are capped to keep malformed graph data from exhausting
    /// memory.
    pub fn detect_rgb(&self, rgb: &[u8], width: usize, height: usize, score_threshold: f32, nms_threshold: f32) -> Result<Vec<Detection>> {
        if !score_threshold.is_finite()
            || !nms_threshold.is_finite()
            || !(0.0..=1.0).contains(&score_threshold)
            || !(0.0..=1.0).contains(&nms_threshold)
        {
            return Err(Error::Model("detection thresholds must be finite values in [0, 1]".into()));
        }
        let input = self.preprocess_rgb(rgb, width, height)?;
        let outputs = self.forward(&input)?;
        decode_and_nms(outputs, width, height, score_threshold, nms_threshold)
    }

    fn execute(&self, node: &Node, values: &HashMap<String, Tensor>) -> Result<Tensor> {
        let input = |index: usize| -> Result<&Tensor> {
            let name = node.inputs.get(index).ok_or_else(|| Error::Model(format!("{} has missing input {index}", node.name)))?;
            values.get(name).ok_or_else(|| Error::Model(format!("{} references unavailable tensor {name}", node.name)))
        };
        let result =
            match node.op.as_str() {
                "Conv" => {
                    let x = input(0)?;
                    let weight = self.float_tensor(node.inputs.get(1).ok_or_else(|| Error::Model("Conv has no weight".into()))?)?;
                    let bias = match node.inputs.get(2) {
                        Some(name) => Some(self.float_tensor(name)?),
                        None => None,
                    };
                    let pads = node.attr_ints("pads")?;
                    let strides = node.attr_ints("strides")?;
                    let dilations = node.attr_ints("dilations")?;
                    let group = node.attr_int("group")?.unwrap_or(1);
                    let pad = symmetric_pair(&pads, "pads")?;
                    let stride = symmetric_pair(&strides, "strides")?;
                    let dilation = symmetric_pair(&dilations, "dilations")?;
                    if group == 0 || group > 64 {
                        return Err(Error::Model(format!("unsupported Conv group {group}")));
                    }
                    let y = x.conv2d(weight, pad, stride, dilation, group)?;
                    match bias {
                        Some(bias) => Ok(y.broadcast_add(&bias.reshape((1, bias.dim(0)?, 1, 1))?)?),
                        None => Ok(y),
                    }
                }
                "Relu" => Ok(input(0)?.relu()?),
                "MaxPool" => {
                    let kernel = node.attr_ints("kernel_shape")?;
                    let strides = node.attr_ints("strides")?;
                    let pads = node.attr_ints("pads")?;
                    if pads.iter().any(|v| *v != 0) || symmetric_pair(&kernel, "kernel_shape")? == 0 {
                        return Err(Error::Model("unsupported MaxPool padding or kernel".into()));
                    }
                    Ok(input(0)?.max_pool2d_with_stride(symmetric_pair(&kernel, "kernel_shape")?, symmetric_pair(&strides, "strides")?)?)
                }
                "Resize" => {
                    let x = input(0)?;
                    let scale_name = node.inputs.get(2).ok_or_else(|| Error::Model("Resize has no scales".into()))?;
                    let scale = self.float_initializer(scale_name)?;
                    let data = scale.data.as_slice();
                    if data.len() != 4
                        || data.get(0) != Some(&1.0)
                        || data.get(1) != Some(&1.0)
                        || !data.get(2).is_some_and(|v| *v >= 1.0 && v.fract() == 0.0)
                        || data.get(2) != data.get(3)
                    {
                        return Err(Error::Model("Resize scales are not bounded nearest-neighbour 2x".into()));
                    }
                    let (_, _, h, w) = x.dims4()?;
                    let scale = *data.get(2).ok_or_else(|| Error::Model("Resize scale missing".into()))? as usize;
                    let h = h.checked_mul(scale).ok_or_else(|| Error::Model("Resize height overflow".into()))?;
                    let w = w.checked_mul(scale).ok_or_else(|| Error::Model("Resize width overflow".into()))?;
                    if h > INPUT_SIDE || w > INPUT_SIDE {
                        return Err(Error::Model("Resize output exceeds fixed input bound".into()));
                    }
                    Ok(x.upsample_nearest2d(h, w)?)
                }
                "Add" => Ok(input(0)?.broadcast_add(input(1)?)?),
                "Transpose" => {
                    let perm = node.attr_ints("perm")?;
                    if perm != [0, 2, 3, 1] {
                        return Err(Error::Model("unsupported Transpose permutation".into()));
                    }
                    Ok(input(0)?.permute((0, 2, 3, 1))?)
                }
                "Reshape" => {
                    let shape_name = node.inputs.get(1).ok_or_else(|| Error::Model("Reshape has no shape".into()))?;
                    let shape = self.int_initializer(shape_name)?;
                    let source = input(0)?;
                    let source_elements = source.dims().iter().try_fold(1usize, |acc, dim| {
                        acc.checked_mul(*dim).ok_or_else(|| Error::Model("Reshape source element count overflow".into()))
                    })?;
                    let mut dims = Vec::with_capacity(shape.int_data.len());
                    let mut inferred = None;
                    for value in &shape.int_data {
                        if *value == -1 && inferred.is_none() {
                            inferred = Some(dims.len());
                            dims.push(0);
                        } else if *value <= 0 {
                            return Err(Error::Model("Reshape has unsupported nonpositive dimension".into()));
                        } else {
                            dims.push(usize::try_from(*value).map_err(|_| Error::Model("Reshape dimension overflow".into()))?);
                        }
                    }
                    let known = dims.iter().filter(|dim| **dim != 0).try_fold(1usize, |acc, dim| {
                        acc.checked_mul(*dim).ok_or_else(|| Error::Model("Reshape dimension product overflow".into()))
                    })?;
                    if let Some(index) = inferred {
                        if known == 0 || source_elements == 0 || source_elements % known != 0 {
                            return Err(Error::Model("Reshape inferred dimension is invalid".into()));
                        }
                        let inferred_dim = source_elements / known;
                        dims[index] = inferred_dim;
                    } else if known != source_elements {
                        return Err(Error::Model("Reshape dimensions do not match source".into()));
                    }
                    if dims.iter().try_fold(1usize, |acc, dim| acc.checked_mul(*dim)).is_none_or(|elements| elements > MAX_TENSOR_ELEMENTS) {
                        return Err(Error::Model("Reshape output exceeds element bound".into()));
                    }
                    Ok(source.reshape(dims.as_slice())?)
                }
                "Sigmoid" => Ok(candle_nn::ops::sigmoid(input(0)?)?),
                other => Err(Error::Model(format!("unsupported YuNet operator {other}"))),
            }?;
        Ok(result)
    }

    fn float_initializer(&self, name: &str) -> Result<&Initializer> {
        let initializer = self.initializers.get(name).ok_or_else(|| Error::Model(format!("missing initializer {name}")))?;
        if initializer.data_type != 1 {
            return Err(Error::Model(format!("initializer {name} is not FLOAT")));
        }
        Ok(initializer)
    }

    fn float_tensor(&self, name: &str) -> Result<&Tensor> {
        self.float_initializer(name)?;
        self.tensors.get(name).ok_or_else(|| Error::Model(format!("missing device tensor {name}")))
    }

    fn int_initializer(&self, name: &str) -> Result<&Initializer> {
        let initializer = self.initializers.get(name).ok_or_else(|| Error::Model(format!("missing initializer {name}")))?;
        if initializer.data_type != 7 {
            return Err(Error::Model(format!("initializer {name} is not INT64")));
        }
        Ok(initializer)
    }
}

fn symmetric_pair(values: &[i64], label: &str) -> Result<usize> {
    let symmetric = match values {
        [first, second] => first == second,
        [top, left, bottom, right] => top == bottom && left == right && top == left,
        _ => false,
    };
    if !symmetric || values.first().is_some_and(|v| *v < 0) {
        return Err(Error::Model(format!("unsupported {label} attribute")));
    }
    usize::try_from(*values.first().ok_or_else(|| Error::Model(format!("missing {label}")))?).map_err(|_| Error::Model(format!("{label} overflow")))
}

fn validate_input_tensor(input: &Tensor) -> Result<()> {
    if input.dtype() != DType::F32 || input.dims() != [1, 3, INPUT_SIDE, INPUT_SIDE] {
        return Err(Error::Model("YuNet input must be finite F32 [1,3,640,640]".into()));
    }
    ensure_finite(input)
}

fn ensure_finite(tensor: &Tensor) -> Result<()> {
    let values = tensor.to_vec1::<f32>().or_else(|_| tensor.flatten_all()?.to_vec1::<f32>())?;
    if values.iter().any(|v| !v.is_finite()) {
        return Err(Error::Model("YuNet tensor contains nonfinite values".into()));
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Initializer {
    data_type: i32,
    dims: Vec<usize>,
    data: Vec<f32>,
    int_data: Vec<i64>,
}

#[derive(Clone, Debug)]
struct Node {
    name: String,
    op: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    attributes: Vec<Attribute>,
}

impl Node {
    fn attr_ints(&self, name: &str) -> Result<Vec<i64>> {
        self.attributes
            .iter()
            .find(|attr| attr.name == name)
            .and_then(|attr| attr.ints.clone())
            .ok_or_else(|| Error::Model(format!("{} lacks {name}", self.name)))
    }

    fn attr_int(&self, name: &str) -> Result<Option<usize>> {
        let Some(attr) = self.attributes.iter().find(|attr| attr.name == name) else {
            return Ok(None);
        };
        let Some(value) = attr.integer else {
            return Err(Error::Model(format!("{} has non-integer {name}", self.name)));
        };
        Ok(Some(usize::try_from(value).map_err(|_| Error::Model(format!("{} {name} overflows", self.name)))?))
    }
}

#[derive(Clone, Debug)]
struct Attribute {
    name: String,
    integer: Option<i64>,
    ints: Option<Vec<i64>>,
}

#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    initializers: HashMap<String, Initializer>,
    input_shape: Vec<usize>,
    outputs: Vec<String>,
}

impl Graph {
    fn parse(bytes: &[u8]) -> Result<Self> {
        let mut reader = ProtoReader::new(bytes);
        let mut graph = None;
        while let Some(field) = reader.next()? {
            if field.number == 7 && field.wire == 2 {
                graph = Some(parse_graph(field.bytes)?);
            }
        }
        graph.ok_or_else(|| Error::Model("ONNX model has no graph".into()))
    }

    fn validate(&self) -> Result<()> {
        if self.nodes.len() != 106
            || self.initializers.len() != 112
            || self.input_shape != [1, 3, INPUT_SIDE, INPUT_SIDE]
            || self.outputs != EXPECTED_OUTPUTS
        {
            return Err(Error::Model("YuNet graph fingerprint does not match pinned 2023mar artifact".into()));
        }
        let mut counts = HashMap::<&str, usize>::new();
        for node in &self.nodes {
            *counts.entry(node.op.as_str()).or_default() += 1;
        }
        for (op, expected) in
            [("Conv", 53), ("MaxPool", 4), ("Relu", 15), ("Resize", 2), ("Add", 2), ("Transpose", 12), ("Reshape", 12), ("Sigmoid", 6)]
        {
            if counts.get(op).copied().unwrap_or(0) != expected {
                return Err(Error::Model(format!("YuNet {op} count mismatch")));
            }
        }
        let mut known = HashSet::from([String::from("input")]);
        known.extend(self.initializers.keys().cloned());
        let mut produced = HashSet::new();
        for node in &self.nodes {
            if node.inputs.iter().any(|name| !known.contains(name)) {
                return Err(Error::Model(format!("{} references unknown tensor", node.name)));
            }
            for output in &node.outputs {
                if output.is_empty() || !produced.insert(output.clone()) {
                    return Err(Error::Model(format!("duplicate or empty output at {}", node.name)));
                }
                known.insert(output.clone());
            }
        }
        if EXPECTED_OUTPUTS.iter().any(|name| !produced.contains(*name)) {
            return Err(Error::Model("YuNet graph outputs are incomplete".into()));
        }
        let float_elements: usize = self.initializers.values().filter(|i| i.data_type == 1).map(|i| i.data.len()).sum();
        if float_elements != 53_112 || self.initializers.values().filter(|i| i.data_type == 7).count() != 3 {
            return Err(Error::Model("YuNet initializer fingerprint mismatch".into()));
        }
        for name in ["290", "362", "395"] {
            let init = self.initializers.get(name).ok_or_else(|| Error::Model(format!("missing shape initializer {name}")))?;
            if init.data_type != 7 || init.int_data.len() != 3 {
                return Err(Error::Model(format!("invalid shape initializer {name}")));
            }
        }
        Ok(())
    }
}

fn parse_graph(bytes: &[u8]) -> Result<Graph> {
    let mut reader = ProtoReader::new(bytes);
    let mut nodes = Vec::new();
    let mut initializers = HashMap::new();
    let mut input_shape = Vec::new();
    let mut outputs = Vec::new();
    while let Some(field) = reader.next()? {
        match (field.number, field.wire) {
            (1, 2) => {
                if nodes.len() >= 106 {
                    return Err(Error::Model("too many YuNet nodes".into()));
                }
                nodes.push(parse_node(field.bytes)?);
            }
            (5, 2) => {
                if initializers.len() >= 112 {
                    return Err(Error::Model("too many YuNet initializers".into()));
                }
                let init = parse_initializer(field.bytes)?;
                if initializers.insert(init.0.clone(), init.1).is_some() {
                    return Err(Error::Model("duplicate YuNet initializer".into()));
                }
            }
            (11, 2) => {
                let (name, shape) = parse_value_info(field.bytes)?;
                if name == "input" {
                    input_shape = shape;
                }
            }
            (12, 2) => outputs.push(parse_value_info(field.bytes)?.0),
            _ => {}
        }
    }
    Ok(Graph { nodes, initializers, input_shape, outputs })
}

fn parse_node(bytes: &[u8]) -> Result<Node> {
    let mut reader = ProtoReader::new(bytes);
    let mut name = String::new();
    let mut op = String::new();
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut attributes = Vec::new();
    while let Some(field) = reader.next()? {
        match (field.number, field.wire) {
            (1, 2) => inputs.push(parse_string(field.bytes, "node input")?),
            (2, 2) => outputs.push(parse_string(field.bytes, "node output")?),
            (3, 2) => name = parse_string(field.bytes, "node name")?,
            (4, 2) => op = parse_string(field.bytes, "node operator")?,
            (5, 2) => attributes.push(parse_attribute(field.bytes)?),
            _ => {}
        }
    }
    if name.is_empty() || op.is_empty() || outputs.len() != 1 || inputs.is_empty() {
        return Err(Error::Model("malformed YuNet node".into()));
    }
    Ok(Node { name, op, inputs, outputs, attributes })
}

fn parse_attribute(bytes: &[u8]) -> Result<Attribute> {
    let mut reader = ProtoReader::new(bytes);
    let mut name = String::new();
    let mut integer = None;
    let mut ints = None;
    while let Some(field) = reader.next()? {
        match (field.number, field.wire) {
            (1, 2) => name = parse_string(field.bytes, "attribute name")?,
            (3, 0) => integer = Some(decode_varint(field.bytes)? as i64),
            (8, 2) => ints = Some(parse_packed_i64(field.bytes)?),
            _ => {}
        }
    }
    if name.is_empty() {
        return Err(Error::Model("malformed YuNet attribute".into()));
    }
    Ok(Attribute { name, integer, ints })
}

fn parse_initializer(bytes: &[u8]) -> Result<(String, Initializer)> {
    let mut reader = ProtoReader::new(bytes);
    let mut dims_i64 = Vec::new();
    let mut data_type = None;
    let mut raw_data = None;
    let mut float_data = Vec::new();
    let mut int_data = Vec::new();
    let mut name = String::new();
    while let Some(field) = reader.next()? {
        match (field.number, field.wire) {
            (1, 0) => dims_i64.push(decode_varint(field.bytes)? as i64),
            (1, 2) => dims_i64.extend(parse_packed_i64(field.bytes)?),
            (2, 0) => data_type = Some(i32::try_from(decode_varint(field.bytes)?).map_err(|_| Error::Model("initializer dtype overflow".into()))?),
            (4, 5) => float_data.push(f32::from_le_bytes(field.bytes.try_into().map_err(|_| Error::Model("invalid float initializer".into()))?)),
            (4, 2) => float_data.extend(parse_packed_f32(field.bytes)?),
            (7, 0) => int_data.push(decode_varint(field.bytes)? as i64),
            (7, 2) => int_data.extend(parse_packed_i64(field.bytes)?),
            (8, 2) => name = parse_string(field.bytes, "initializer name")?,
            (9, 2) => raw_data = Some(field.bytes),
            _ => {}
        }
    }
    if name.is_empty() || dims_i64.iter().any(|dim| *dim < 0) {
        return Err(Error::Model("malformed YuNet initializer".into()));
    }
    let dims = dims_i64
        .into_iter()
        .map(|dim| usize::try_from(dim).map_err(|_| Error::Model("initializer dimension overflow".into())))
        .collect::<Result<Vec<_>>>()?;
    let expected =
        dims.iter().try_fold(1usize, |acc, dim| acc.checked_mul(*dim)).ok_or_else(|| Error::Model("initializer element count overflow".into()))?;
    if expected > MAX_TENSOR_ELEMENTS {
        return Err(Error::Model("initializer exceeds bounded element count".into()));
    }
    let data_type = data_type.ok_or_else(|| Error::Model(format!("initializer {name} has no dtype")))?;
    let (data, int_data) = match data_type {
        1 => {
            let data = if let Some(raw) = raw_data { parse_raw_f32(raw)? } else { float_data };
            if data.len() != expected {
                return Err(Error::Model(format!("initializer {name} float length mismatch")));
            }
            if data.iter().any(|v| !v.is_finite()) {
                return Err(Error::Model(format!("initializer {name} is nonfinite")));
            }
            (data, Vec::new())
        }
        7 => {
            let ints = if let Some(raw) = raw_data { parse_raw_i64(raw)? } else { int_data };
            if ints.len() != expected {
                return Err(Error::Model(format!("initializer {name} int length mismatch")));
            }
            (Vec::new(), ints)
        }
        _ => return Err(Error::Model(format!("initializer {name} uses unsupported dtype {data_type}"))),
    };
    Ok((name, Initializer { data_type, dims, data, int_data }))
}

fn parse_value_info(bytes: &[u8]) -> Result<(String, Vec<usize>)> {
    let mut reader = ProtoReader::new(bytes);
    let mut name = String::new();
    let mut shape = Vec::new();
    while let Some(field) = reader.next()? {
        match (field.number, field.wire) {
            (1, 2) => name = parse_string(field.bytes, "value name")?,
            (2, 2) => shape = parse_type(field.bytes)?,
            _ => {}
        }
    }
    Ok((name, shape))
}

fn parse_type(bytes: &[u8]) -> Result<Vec<usize>> {
    let mut reader = ProtoReader::new(bytes);
    while let Some(field) = reader.next()? {
        if field.number == 2 && field.wire == 2 {
            return parse_shape(field.bytes);
        }
    }
    Ok(Vec::new())
}

fn parse_shape(bytes: &[u8]) -> Result<Vec<usize>> {
    let mut reader = ProtoReader::new(bytes);
    let mut shape = Vec::new();
    while let Some(field) = reader.next()? {
        if field.number != 1 || field.wire != 2 {
            continue;
        }
        let mut dim_reader = ProtoReader::new(field.bytes);
        let mut value = None;
        while let Some(dim) = dim_reader.next()? {
            if dim.number == 1 && dim.wire == 0 {
                value = Some(usize::try_from(decode_varint(dim.bytes)?).map_err(|_| Error::Model("shape dimension overflow".into()))?);
            }
        }
        shape.push(value.ok_or_else(|| Error::Model("YuNet graph has symbolic dimension".into()))?);
    }
    Ok(shape)
}

fn parse_string(bytes: &[u8], label: &str) -> Result<String> {
    if bytes.len() > MAX_PROTO_STRING {
        return Err(Error::Model(format!("{label} is too long")));
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| Error::Model(format!("{label} is not UTF-8")))
}

fn parse_raw_f32(bytes: &[u8]) -> Result<Vec<f32>> {
    if bytes.len() % 4 != 0 || bytes.len() / 4 > MAX_TENSOR_ELEMENTS {
        return Err(Error::Model("invalid FLOAT raw_data length".into()));
    }
    bytes.chunks_exact(4).map(|chunk| Ok(f32::from_le_bytes(chunk.try_into().map_err(|_| Error::Model("invalid FLOAT raw_data".into()))?))).collect()
}

fn parse_raw_i64(bytes: &[u8]) -> Result<Vec<i64>> {
    if bytes.len() % 8 != 0 || bytes.len() / 8 > MAX_TENSOR_ELEMENTS {
        return Err(Error::Model("invalid INT64 raw_data length".into()));
    }
    bytes.chunks_exact(8).map(|chunk| Ok(i64::from_le_bytes(chunk.try_into().map_err(|_| Error::Model("invalid INT64 raw_data".into()))?))).collect()
}

fn parse_packed_f32(bytes: &[u8]) -> Result<Vec<f32>> {
    parse_raw_f32(bytes)
}

fn parse_packed_i64(bytes: &[u8]) -> Result<Vec<i64>> {
    let mut pos = 0;
    let mut values = Vec::new();
    while pos < bytes.len() {
        values.push(decode_varint_from(bytes, &mut pos)? as i64);
        if values.len() > 1024 {
            return Err(Error::Model("packed INT64 field exceeds bound".into()));
        }
    }
    Ok(values)
}

struct ProtoField<'a> {
    number: u32,
    wire: u8,
    bytes: &'a [u8],
}

struct ProtoReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ProtoReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn next(&mut self) -> Result<Option<ProtoField<'a>>> {
        if self.position == self.bytes.len() {
            return Ok(None);
        }
        let key = self.read_varint()?;
        let number = u32::try_from(key >> 3).map_err(|_| Error::Model("protobuf field number overflow".into()))?;
        let wire = u8::try_from(key & 7).map_err(|_| Error::Model("protobuf wire type overflow".into()))?;
        if number == 0 {
            return Err(Error::Model("protobuf field number is zero".into()));
        }
        let bytes = match wire {
            0 => {
                let start = self.position;
                let _ = self.read_varint()?;
                self.bytes.get(start..self.position).ok_or_else(|| Error::Model("protobuf varint range invalid".into()))?
            }
            1 => self.take(8)?,
            2 => {
                let length = usize::try_from(self.read_varint()?).map_err(|_| Error::Model("protobuf length overflow".into()))?;
                self.take(length)?
            }
            5 => self.take(4)?,
            _ => return Err(Error::Model(format!("unsupported protobuf wire type {wire}"))),
        };
        Ok(Some(ProtoField { number, wire, bytes }))
    }

    fn read_varint(&mut self) -> Result<u64> {
        let start = self.position;
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = *self.bytes.get(self.position).ok_or_else(|| Error::Model("truncated protobuf varint".into()))?;
            self.position = self.position.checked_add(1).ok_or_else(|| Error::Model("protobuf offset overflow".into()))?;
            let part = u64::from(byte & 0x7f);
            if shift == 63 && part > 1 {
                return Err(Error::Model("protobuf varint overflow".into()));
            }
            value |= part.checked_shl(shift).ok_or_else(|| Error::Model("protobuf varint overflow".into()))?;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        let _ = start;
        Err(Error::Model("protobuf varint is too long".into()))
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self.position.checked_add(length).ok_or_else(|| Error::Model("protobuf length overflow".into()))?;
        let bytes = self.bytes.get(self.position..end).ok_or_else(|| Error::Model("truncated protobuf field".into()))?;
        self.position = end;
        Ok(bytes)
    }
}

fn decode_varint(bytes: &[u8]) -> Result<u64> {
    let mut position = 0;
    decode_varint_from(bytes, &mut position)
}

fn decode_varint_from(bytes: &[u8], position: &mut usize) -> Result<u64> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let byte = *bytes.get(*position).ok_or_else(|| Error::Model("truncated protobuf varint".into()))?;
        *position = (*position).checked_add(1).ok_or_else(|| Error::Model("protobuf offset overflow".into()))?;
        let part = u64::from(byte & 0x7f);
        if shift == 63 && part > 1 {
            return Err(Error::Model("protobuf varint overflow".into()));
        }
        value |= part.checked_shl(shift).ok_or_else(|| Error::Model("protobuf varint overflow".into()))?;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(Error::Model("protobuf varint is too long".into()))
}

fn preprocess_rgb(rgb: &[u8], width: usize, height: usize, policy: Preprocess, device: &Device) -> Result<Tensor> {
    let pixels = width.checked_mul(height).ok_or_else(|| Error::Model("image dimensions overflow".into()))?;
    let expected = pixels.checked_mul(3).ok_or_else(|| Error::Model("image buffer length overflow".into()))?;
    if width == 0 || height == 0 || expected != rgb.len() || width > 16_384 || height > 16_384 {
        return Err(Error::Model("RGB image dimensions or data length are invalid".into()));
    }
    let (resized_w, resized_h) = resize_shape(width, height)?;
    let plane = INPUT_SIDE.checked_mul(INPUT_SIDE).ok_or_else(|| Error::Model("input plane overflow".into()))?;
    let total = plane.checked_mul(3).ok_or_else(|| Error::Model("input tensor size overflow".into()))?;
    let mut data = vec![0.0f32; total];
    for y in 0..resized_h {
        let src_y = y.saturating_mul(height) / resized_h;
        for x in 0..resized_w {
            let src_x = x.saturating_mul(width) / resized_w;
            let source = src_y
                .checked_mul(width)
                .and_then(|v| v.checked_add(src_x))
                .and_then(|v| v.checked_mul(3))
                .ok_or_else(|| Error::Model("image sample offset overflow".into()))?;
            let dst = y.checked_mul(INPUT_SIDE).and_then(|v| v.checked_add(x)).ok_or_else(|| Error::Model("input sample offset overflow".into()))?;
            for channel in 0..3 {
                let output_channel = if policy.swap_rb { 2 - channel } else { channel };
                let value = *rgb.get(source + channel).ok_or_else(|| Error::Model("RGB sample offset out of range".into()))? as f32;
                let slot = output_channel
                    .checked_mul(plane)
                    .and_then(|v| v.checked_add(dst))
                    .ok_or_else(|| Error::Model("input channel offset overflow".into()))?;
                let destination = data.get_mut(slot).ok_or_else(|| Error::Model("input channel offset out of range".into()))?;
                *destination = (value - policy.mean[channel]) * policy.scale;
            }
        }
    }
    Tensor::from_vec(data, (1, 3, INPUT_SIDE, INPUT_SIDE), device).map_err(Error::from)
}

fn resize_shape(width: usize, height: usize) -> Result<(usize, usize)> {
    if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
        return Err(Error::Model("RGB image dimensions are invalid".into()));
    }
    let scale = (INPUT_SIDE as f64 / width as f64).min(INPUT_SIDE as f64 / height as f64);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(Error::Model("image resize scale is invalid".into()));
    }
    let resized_w = width as f64 * scale;
    let resized_h = height as f64 * scale;
    if !resized_w.is_finite() || !resized_h.is_finite() || resized_w < 1.0 || resized_h < 1.0 {
        return Err(Error::Model("image resize dimensions are invalid".into()));
    }
    let resized_w = (resized_w.round() as usize).clamp(1, INPUT_SIDE);
    let resized_h = (resized_h.round() as usize).clamp(1, INPUT_SIDE);
    Ok((resized_w, resized_h))
}

fn decode_and_nms(outputs: RawOutputs, width: usize, height: usize, score_threshold: f32, nms_threshold: f32) -> Result<Vec<Detection>> {
    let (resized_w, resized_h) = resize_shape(width, height)?;
    let mut candidates = Vec::new();
    let strides = [8usize, 16, 32];
    for level in 0..3 {
        let stride = *strides.get(level).ok_or_else(|| Error::Model("YuNet stride index out of range".into()))?;
        let cells = (INPUT_SIDE / stride).checked_mul(INPUT_SIDE / stride).ok_or_else(|| Error::Model("YuNet cell count overflow".into()))?;
        let cls = output_values(outputs.cls.get(level).ok_or_else(|| Error::Model("missing cls output".into()))?, cells, 1)?;
        let obj = output_values(outputs.obj.get(level).ok_or_else(|| Error::Model("missing obj output".into()))?, cells, 1)?;
        let bbox = output_values(outputs.bbox.get(level).ok_or_else(|| Error::Model("missing bbox output".into()))?, cells, 4)?;
        let keypoints = output_values(outputs.keypoints.get(level).ok_or_else(|| Error::Model("missing keypoint output".into()))?, cells, 10)?;
        for cell in 0..cells {
            let class_score = *cls.get(cell).ok_or_else(|| Error::Model("cls output index out of range".into()))?;
            let object_score = *obj.get(cell).ok_or_else(|| Error::Model("obj output index out of range".into()))?;
            if !(0.0..=1.0).contains(&class_score) || !(0.0..=1.0).contains(&object_score) {
                return Err(Error::Model("YuNet sigmoid output is outside [0, 1]".into()));
            }
            let score = (class_score * object_score).sqrt();
            if !score.is_finite() {
                return Err(Error::Model("YuNet output contains nonfinite score".into()));
            }
            if score < score_threshold {
                continue;
            }
            let bx = cell % (INPUT_SIDE / stride);
            let by = cell / (INPUT_SIDE / stride);
            let offset = cell.checked_mul(4).ok_or_else(|| Error::Model("bbox output offset overflow".into()))?;
            let dx = *bbox.get(offset).ok_or_else(|| Error::Model("bbox output is truncated".into()))?;
            let dy = *bbox.get(offset + 1).ok_or_else(|| Error::Model("bbox output is truncated".into()))?;
            let dw = *bbox.get(offset + 2).ok_or_else(|| Error::Model("bbox output is truncated".into()))?;
            let dh = *bbox.get(offset + 3).ok_or_else(|| Error::Model("bbox output is truncated".into()))?;
            if [dx, dy, dw, dh].iter().any(|value| !value.is_finite() || value.abs() > 10_000.0) {
                return Err(Error::Model("YuNet output contains out-of-range box values".into()));
            }
            let anchor_x = bx as f32 * stride as f32;
            let anchor_y = by as f32 * stride as f32;
            let center_x = anchor_x + dx * stride as f32;
            let center_y = anchor_y + dy * stride as f32;
            let box_width = dw.exp() * stride as f32;
            let box_height = dh.exp() * stride as f32;
            if ![center_x, center_y, box_width, box_height].iter().all(|value| value.is_finite()) || box_width <= 0.0 || box_height <= 0.0 {
                return Err(Error::Model("YuNet output contains invalid box values".into()));
            }
            let x0 = (center_x - box_width * 0.5).clamp(0.0, INPUT_SIDE as f32);
            let y0 = (center_y - box_height * 0.5).clamp(0.0, INPUT_SIDE as f32);
            let x1 = (center_x + box_width * 0.5).clamp(0.0, INPUT_SIDE as f32);
            let y1 = (center_y + box_height * 0.5).clamp(0.0, INPUT_SIDE as f32);
            if x1 <= x0 || y1 <= y0 {
                return Err(Error::Model("YuNet output contains a degenerate box".into()));
            }
            let sx = resized_w as f32 / width as f32;
            let sy = resized_h as f32 / height as f32;
            let mut keypoint_values = [[0.0f32; 2]; 5];
            let keypoint_offset = cell.checked_mul(10).ok_or_else(|| Error::Model("keypoint output offset overflow".into()))?;
            for point in 0..5 {
                let point_offset = keypoint_offset
                    .checked_add(point.checked_mul(2).ok_or_else(|| Error::Model("keypoint offset overflow".into()))?)
                    .ok_or_else(|| Error::Model("keypoint offset overflow".into()))?;
                let px = anchor_x + *keypoints.get(point_offset).ok_or_else(|| Error::Model("keypoint output is truncated".into()))? * stride as f32;
                let py =
                    anchor_y + *keypoints.get(point_offset + 1).ok_or_else(|| Error::Model("keypoint output is truncated".into()))? * stride as f32;
                if !px.is_finite() || !py.is_finite() || px.abs() > 1_000_000.0 || py.abs() > 1_000_000.0 {
                    return Err(Error::Model("YuNet output contains invalid keypoint values".into()));
                }
                if let Some(point_value) = keypoint_values.get_mut(point) {
                    *point_value = [px / sx, py / sy];
                }
            }
            candidates.push(Detection { score, bbox: [x0 / sx, y0 / sy, x1 / sx, y1 / sy], keypoints: keypoint_values });
            if candidates.len() > MAX_CANDIDATES {
                return Err(Error::Model("YuNet candidate count exceeds bound".into()));
            }
        }
    }
    candidates.sort_by(|left, right| right.score.partial_cmp(&left.score).unwrap_or(Ordering::Equal));
    let mut selected = Vec::with_capacity(candidates.len().min(MAX_DETECTIONS));
    for candidate in candidates {
        if selected.iter().all(|kept: &Detection| iou(&kept.bbox, &candidate.bbox) <= nms_threshold) {
            selected.push(candidate);
            if selected.len() == MAX_DETECTIONS {
                break;
            }
        }
    }
    Ok(selected)
}

fn output_values(output: &Tensor, cells: usize, channels: usize) -> Result<Vec<f32>> {
    let expected = cells.checked_mul(channels).ok_or_else(|| Error::Model("YuNet output element count overflow".into()))?;
    if output.dims() != [1, cells, channels] {
        return Err(Error::Model(format!("YuNet output shape {:?} does not match [1,{cells},{channels}]", output.dims())));
    }
    let values = output.flatten_all()?.to_vec1::<f32>()?;
    if values.len() != expected || values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Model("YuNet output data is malformed or nonfinite".into()));
    }
    Ok(values)
}

fn iou(left: &[f32; 4], right: &[f32; 4]) -> f32 {
    let x0 = left[0].max(right[0]);
    let y0 = left[1].max(right[1]);
    let x1 = left[2].min(right[2]);
    let y1 = left[3].min(right[3]);
    let intersection = (x1 - x0).max(0.0) * (y1 - y0).max(0.0);
    let left_area = (left[2] - left[0]).max(0.0) * (left[3] - left[1]).max(0.0);
    let right_area = (right[2] - right[0]).max(0.0) * (right[3] - right[1]).max(0.0);
    let union = left_area + right_area - intersection;
    if union <= 0.0 || !union.is_finite() { 0.0 } else { (intersection / union).clamp(0.0, 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocessing_rejects_malformed_rgb_lengths() {
        let result = preprocess_rgb(&[0; 2], 1, 1, Preprocess::default(), &Device::Cpu);
        assert!(result.is_err(), "short RGB must fail");
        if let Err(err) = result {
            assert!(err.to_string().contains("dimensions or data length"));
        }
    }

    #[test]
    fn preprocessing_rejects_empty_dimensions_without_allocation() {
        let result = preprocess_rgb(&[], 0, 0, Preprocess::default(), &Device::Cpu);
        assert!(result.is_err(), "empty image must fail");
        if let Err(err) = result {
            assert!(err.to_string().contains("dimensions or data length"));
        }
    }

    #[test]
    fn iou_handles_degenerate_boxes() {
        assert_eq!(iou(&[0.0, 0.0, 0.0, 0.0], &[0.0, 0.0, 1.0, 1.0]), 0.0);
    }

    #[test]
    fn output_values_reject_nonfinite_tensor_data() {
        let result = Tensor::from_vec(vec![f32::NAN], (1, 1, 1), &Device::Cpu);
        assert!(result.is_ok(), "test tensor construction failed");
        if let Ok(tensor) = result {
            assert!(output_values(&tensor, 1, 1).is_err());
        }
    }
}
