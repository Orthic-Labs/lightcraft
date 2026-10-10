//! Fixed graph contracts for the pinned MediaPipe Face Landmarker v1 bundle.
//!
//! These descriptors contain architecture metadata only. Artifact bytes are verified
//! before constants are loaded; no FlatBuffer or operator-code parser runs in production.
//! Source-only experiment: no reference parity or culling quality is qualified.

#![cfg(not(target_arch = "wasm32"))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum GraphId {
    Detector,
    Landmarks,
    Blendshapes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum StorageType {
    Float32,
    Float16,
    Int32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TensorSpec {
    pub name: &'static str,
    pub shape: &'static [usize],
    pub dtype: StorageType,
    /// Absolute byte offset in the exact full `.task` bundle; absent for runtime tensors.
    pub offset: Option<usize>,
    pub byte_len: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EntrySpec {
    pub name: &'static str,
    pub offset: usize,
    pub byte_len: usize,
    pub sha256: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// Preserve official fused-activation tags even when this pinned graph uses only None/Relu.
#[allow(dead_code)]
pub(crate) enum Activation {
    None,
    Relu,
    ReluN1To1,
    Relu6,
    Tanh,
    SignBit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Padding {
    Same,
    Valid,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Op {
    Conv2d {
        stride_h: usize,
        stride_w: usize,
        dilation_h: usize,
        dilation_w: usize,
        padding: Padding,
        activation: Activation,
    },
    DepthwiseConv2d {
        stride_h: usize,
        stride_w: usize,
        dilation_h: usize,
        dilation_w: usize,
        depth_multiplier: usize,
        padding: Padding,
        activation: Activation,
    },
    MaxPool2d {
        stride_h: usize,
        stride_w: usize,
        filter_h: usize,
        filter_w: usize,
        padding: Padding,
        activation: Activation,
    },
    Add {
        activation: Activation,
    },
    Sub {
        activation: Activation,
    },
    Mul {
        activation: Activation,
    },
    Div {
        activation: Activation,
    },
    Relu,
    Logistic,
    Prelu,
    Neg,
    Sqrt,
    Rsqrt,
    SquaredDifference,
    Dequantize,
    Pad,
    Reshape {
        new_shape: &'static [i32],
    },
    Concat {
        axis: i32,
        activation: Activation,
    },
    Mean {
        keep_dims: bool,
    },
    Sum {
        keep_dims: bool,
    },
    Transpose,
    StridedSlice {
        begin_mask: i32,
        end_mask: i32,
        ellipsis_mask: i32,
        new_axis_mask: i32,
        shrink_axis_mask: i32,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Node {
    pub op: Op,
    pub inputs: &'static [usize],
    pub output: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GraphSpec {
    pub id: GraphId,
    pub name: &'static str,
    pub inputs: &'static [usize],
    pub outputs: &'static [usize],
    pub tensors: &'static [TensorSpec],
    pub nodes: &'static [Node],
}

/// Float16 storage is materialized as Float32 once; integer control tensors stay on CPU.
#[derive(Clone, Debug)]
pub(crate) enum Value {
    Float(candle_core::Tensor),
    Int { values: Vec<i32>, shape: Vec<usize> },
}

impl Value {
    pub(crate) fn float(&self) -> crate::Result<&candle_core::Tensor> {
        match self {
            Self::Float(tensor) => Ok(tensor),
            Self::Int { .. } => Err(crate::Error::Model("MediaPipe operation expected Float32 tensor".into())),
        }
    }

    pub(crate) fn ints(&self) -> crate::Result<(&[i32], &[usize])> {
        match self {
            Self::Int { values, shape } => Ok((values, shape)),
            Self::Float(_) => Err(crate::Error::Model("MediaPipe operation expected Int32 control tensor".into())),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedGraph {
    pub spec: &'static GraphSpec,
    pub constants: Vec<Option<Value>>,
}
