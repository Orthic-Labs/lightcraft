//! Non-convolution operators used by the fixed MediaPipe TFLite graphs.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{DType, Tensor};

use crate::mediapipe_graph::{Activation, Op, TensorSpec, Value};
use crate::{Error, Result};

const MAX_CONTROL_RANK: usize = 8;

pub(crate) fn execute(op: &Op, inputs: &[Value], expected: &TensorSpec) -> Result<Value> {
    let value = match op {
        Op::Conv2d { .. } | Op::DepthwiseConv2d { .. } | Op::MaxPool2d { .. } => {
            return finish_value(crate::mediapipe_conv::execute(op, inputs)?, expected);
        }
        Op::Add { activation } => binary(inputs, "add", |lhs, rhs| lhs.broadcast_add(rhs), *activation)?,
        Op::Sub { activation } => binary(inputs, "sub", |lhs, rhs| lhs.broadcast_sub(rhs), *activation)?,
        Op::Mul { activation } => binary(inputs, "mul", |lhs, rhs| lhs.broadcast_mul(rhs), *activation)?,
        Op::Div { activation } => binary(inputs, "div", |lhs, rhs| lhs.broadcast_div(rhs), *activation)?,
        Op::Relu => unary(inputs, "relu", Tensor::relu)?,
        Op::Logistic => logistic(single_float(inputs, "logistic")?)?,
        Op::Prelu => prelu(inputs)?,
        Op::Neg => unary(inputs, "neg", Tensor::neg)?,
        Op::Sqrt => unary(inputs, "sqrt", Tensor::sqrt)?,
        Op::Rsqrt => unary(inputs, "rsqrt", |tensor| tensor.sqrt()?.recip())?,
        Op::SquaredDifference => binary(inputs, "squared-difference", |lhs, rhs| lhs.broadcast_sub(rhs)?.sqr(), Activation::None)?,
        Op::Dequantize => dequantize(inputs)?,
        Op::Pad => pad(inputs, expected)?,
        Op::Reshape { new_shape } => reshape(inputs, new_shape, expected)?,
        Op::Concat { axis, activation } => concat(inputs, *axis, *activation)?,
        Op::Mean { keep_dims } => reduce(inputs, *keep_dims, false)?,
        Op::Sum { keep_dims } => reduce(inputs, *keep_dims, true)?,
        Op::Transpose => transpose(inputs)?,
        Op::StridedSlice { begin_mask, end_mask, ellipsis_mask, new_axis_mask, shrink_axis_mask } => {
            strided_slice(inputs, *begin_mask, *end_mask, *ellipsis_mask, *new_axis_mask, *shrink_axis_mask)?
        }
    };
    finish(value, expected)
}

pub(crate) fn activate(tensor: &Tensor, activation: Activation) -> Result<Tensor> {
    if tensor.dtype() != DType::F32 {
        return model("MediaPipe activation requires Float32 tensor");
    }
    match activation {
        Activation::None => Ok(tensor.clone()),
        Activation::Relu => Ok(tensor.relu()?),
        Activation::ReluN1To1 => Ok(tensor.clamp(-1f64, 1f64)?),
        Activation::Relu6 => Ok(tensor.clamp(0f64, 6f64)?),
        Activation::Tanh => Ok(tensor.tanh()?),
        // TFLite SIGN_BIT returns one for negative values and zero otherwise.
        Activation::SignBit => Ok(tensor.lt(0f64)?.to_dtype(DType::F32)?),
    }
}

fn model<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::Model(message.into()))
}

fn require_count<'a>(inputs: &'a [Value], count: usize, name: &str) -> Result<&'a [Value]> {
    if inputs.len() != count {
        return model(format!("MediaPipe {name} expects {count} inputs, got {}", inputs.len()));
    }
    Ok(inputs)
}

fn single_float<'a>(inputs: &'a [Value], name: &str) -> Result<&'a Tensor> {
    require_count(inputs, 1, name)?;
    inputs[0].float()
}

fn binary<F>(inputs: &[Value], name: &'static str, operation: F, activation: Activation) -> Result<Tensor>
where
    F: FnOnce(&Tensor, &Tensor) -> candle_core::Result<Tensor>,
{
    require_count(inputs, 2, name)?;
    let lhs = inputs[0].float()?;
    let rhs = inputs[1].float()?;
    if lhs.dtype() != DType::F32 || rhs.dtype() != DType::F32 {
        return model(format!("MediaPipe {name} requires Float32 tensors"));
    }
    let shape = lhs.shape().broadcast_shape_binary_op(rhs.shape(), name).map_err(Error::from)?;
    let output = operation(lhs, rhs)?;
    if output.dims() != shape.dims() {
        return model(format!("MediaPipe {name} returned unexpected broadcast shape {:?}", output.dims()));
    }
    activate(&output, activation)
}

fn unary<F>(inputs: &[Value], name: &str, operation: F) -> Result<Tensor>
where
    F: FnOnce(&Tensor) -> candle_core::Result<Tensor>,
{
    let input = single_float(inputs, name)?;
    if input.dtype() != DType::F32 {
        return model(format!("MediaPipe {name} requires Float32 tensor"));
    }
    Ok(operation(input)?)
}

fn logistic(input: &Tensor) -> Result<Tensor> {
    // This form bounds exponent input by zero, avoiding inf/inf for large finite values.
    let magnitude = input.abs()?;
    let exp = magnitude.neg()?.exp()?;
    let denominator = exp.affine(1., 1.)?;
    let positive = denominator.recip()?;
    let negative = exp.broadcast_div(&denominator)?;
    let mask = input.ge(0f64)?;
    Ok(mask.where_cond(&positive, &negative)?)
}

fn prelu(inputs: &[Value]) -> Result<Tensor> {
    require_count(inputs, 2, "prelu")?;
    let input = inputs[0].float()?;
    let alpha = inputs[1].float()?;
    if input.dtype() != DType::F32 || alpha.dtype() != DType::F32 {
        return model("MediaPipe prelu requires Float32 tensors");
    }
    let shape = input.shape().broadcast_shape_binary_op(alpha.shape(), "prelu").map_err(Error::from)?;
    if shape.dims() != input.dims() {
        return model("MediaPipe prelu alpha would change output shape");
    }
    let zero = input.zeros_like()?;
    let negative = input.broadcast_minimum(&zero)?.broadcast_mul(alpha)?;
    Ok(input.relu()?.broadcast_add(&negative)?)
}

fn dequantize(inputs: &[Value]) -> Result<Tensor> {
    // Artifact loader has already converted Float16 constants to Float32.
    single_float(inputs, "dequantize").cloned()
}

fn pad(inputs: &[Value], expected: &TensorSpec) -> Result<Tensor> {
    require_count(inputs, 2, "pad")?;
    let input = inputs[0].float()?;
    let (padding, padding_shape) = inputs[1].ints()?;
    if input.dtype() != DType::F32 {
        return model("MediaPipe pad requires Float32 input");
    }
    let rank = input.rank();
    if padding_shape != [rank, 2] || padding.len() != rank.checked_mul(2).ok_or_else(|| Error::Model("MediaPipe pad rank overflow".into()))? {
        return model("MediaPipe pad requires Int32 [rank, 2] paddings");
    }
    if expected.dtype != crate::mediapipe_graph::StorageType::Float32 || expected.shape.len() != rank {
        return model("MediaPipe pad output spec is not Float32 with matching rank");
    }
    let mut output_shape = Vec::with_capacity(rank);
    let mut pads = Vec::with_capacity(rank);
    let (pairs, remainder) = padding.as_chunks::<2>();
    if !remainder.is_empty() {
        return model("MediaPipe pad requires Int32 [rank, 2] paddings");
    }
    for (dim, pair) in pairs.iter().enumerate() {
        let left = usize::try_from(pair[0]).map_err(|_| Error::Model("MediaPipe pad has negative padding".into()))?;
        let right = usize::try_from(pair[1]).map_err(|_| Error::Model("MediaPipe pad has negative padding".into()))?;
        let size = input.dims().get(dim).copied().ok_or_else(|| Error::Model("MediaPipe pad dimension is out of range".into()))?;
        let result =
            size.checked_add(left).and_then(|v| v.checked_add(right)).ok_or_else(|| Error::Model("MediaPipe pad output shape overflows".into()))?;
        output_shape.push(result);
        pads.push((left, right));
    }
    if expected.shape != output_shape.as_slice() {
        return model(format!("MediaPipe pad output shape mismatch: expected {:?}, computed {:?}", expected.shape, output_shape));
    }
    let mut output = input.clone();
    for (dim, (left, right)) in pads.into_iter().enumerate() {
        output = output.pad_with_zeros(dim, left, right)?;
    }
    Ok(output)
}

fn reshape(inputs: &[Value], descriptor: &[i32], expected: &TensorSpec) -> Result<Tensor> {
    if inputs.len() != 1 && inputs.len() != 2 {
        return model(format!("MediaPipe reshape expects one Float32 input plus optional shape, got {} inputs", inputs.len()));
    }
    let input = inputs[0].float()?;
    if input.dtype() != DType::F32 {
        return model("MediaPipe reshape requires Float32 input");
    }
    let shape_values = if inputs.len() == 2 {
        let (values, shape) = inputs[1].ints()?;
        validate_control_shape(shape, values.len(), "reshape shape")?;
        values.to_vec()
    } else {
        descriptor.to_vec()
    };
    if shape_values.len() > MAX_CONTROL_RANK || shape_values.is_empty() && descriptor.is_empty() {
        // Empty shape is valid only for a scalar result and is handled below.
        if !shape_values.is_empty() {
            return model("MediaPipe reshape rank exceeds bound");
        }
    }
    let mut output_shape = Vec::with_capacity(shape_values.len());
    let mut hole = None;
    let mut known = 1usize;
    for (index, &dimension) in shape_values.iter().enumerate() {
        if dimension == -1 {
            if hole.replace(index).is_some() {
                return model("MediaPipe reshape has more than one inferred dimension");
            }
            output_shape.push(1);
        } else {
            let dimension = usize::try_from(dimension).map_err(|_| Error::Model("MediaPipe reshape has invalid dimension".into()))?;
            if dimension == 0 {
                return model("MediaPipe reshape has zero dimension");
            }
            known = known.checked_mul(dimension).ok_or_else(|| Error::Model("MediaPipe reshape shape overflows".into()))?;
            output_shape.push(dimension);
        }
    }
    let elements = input.elem_count();
    if let Some(index) = hole {
        if known == 0 || elements % known != 0 {
            return model("MediaPipe reshape inferred dimension does not divide input size");
        }
        output_shape[index] = elements / known;
    } else if known != elements {
        return model("MediaPipe reshape changes element count");
    }
    if expected.shape != output_shape.as_slice() {
        return model(format!("MediaPipe reshape output shape mismatch: expected {:?}, computed {:?}", expected.shape, output_shape));
    }
    Ok(input.reshape(output_shape)?)
}

fn concat(inputs: &[Value], axis: i32, activation: Activation) -> Result<Tensor> {
    if inputs.is_empty() {
        return model("MediaPipe concat requires at least one input");
    }
    let tensors = inputs.iter().map(Value::float).collect::<Result<Vec<_>>>()?;
    if tensors.iter().any(|tensor| tensor.dtype() != DType::F32) {
        return model("MediaPipe concat requires Float32 tensors");
    }
    let rank = tensors[0].rank();
    let axis = normalize_axis(axis, rank, "concat")?;
    let shape = tensors[0].dims();
    for tensor in tensors.iter().skip(1) {
        if tensor.rank() != rank || tensor.dims().iter().enumerate().any(|(dim, size)| dim != axis && *size != shape[dim]) {
            return model("MediaPipe concat has incompatible input shapes");
        }
    }
    activate(&Tensor::cat(&tensors, axis)?, activation)
}

fn reduce(inputs: &[Value], keep_dims: bool, sum: bool) -> Result<Tensor> {
    require_count(inputs, 2, if sum { "sum" } else { "mean" })?;
    let input = inputs[0].float()?;
    let (axes, axes_shape) = inputs[1].ints()?;
    if input.dtype() != DType::F32 {
        return model("MediaPipe reduction requires Float32 input");
    }
    validate_control_shape(axes_shape, axes.len(), "reduction axes")?;
    let normalized = normalize_axes(axes, input.rank(), if sum { "sum" } else { "mean" })?;
    Ok(if sum {
        if keep_dims { input.sum_keepdim(normalized)? } else { input.sum(normalized)? }
    } else if keep_dims {
        input.mean_keepdim(normalized)?
    } else {
        input.mean(normalized)?
    })
}

fn transpose(inputs: &[Value]) -> Result<Tensor> {
    require_count(inputs, 2, "transpose")?;
    let input = inputs[0].float()?;
    let (permutation, shape) = inputs[1].ints()?;
    if input.dtype() != DType::F32 {
        return model("MediaPipe transpose requires Float32 input");
    }
    validate_control_shape(shape, permutation.len(), "transpose permutation")?;
    if permutation.len() != input.rank() {
        return model("MediaPipe transpose permutation rank mismatch");
    }
    let mut permutation_usize = Vec::with_capacity(permutation.len());
    for &axis in permutation {
        let axis = usize::try_from(axis).map_err(|_| Error::Model("MediaPipe transpose has negative axis".into()))?;
        if axis >= input.rank() || permutation_usize.contains(&axis) {
            return model("MediaPipe transpose permutation is not a permutation");
        }
        permutation_usize.push(axis);
    }
    Ok(input.permute(permutation_usize)?)
}

fn strided_slice(inputs: &[Value], begin_mask: i32, end_mask: i32, ellipsis_mask: i32, new_axis_mask: i32, shrink_axis_mask: i32) -> Result<Tensor> {
    require_count(inputs, 4, "strided slice")?;
    let input = inputs[0].float()?;
    if input.dtype() != DType::F32 {
        return model("MediaPipe strided slice requires Float32 input");
    }
    if ellipsis_mask != 0 || new_axis_mask != 0 {
        return model("MediaPipe strided slice ellipsis/new-axis masks are unsupported");
    }
    let (begin, begin_shape) = inputs[1].ints()?;
    let (end, end_shape) = inputs[2].ints()?;
    let (stride, stride_shape) = inputs[3].ints()?;
    validate_control_shape(begin_shape, begin.len(), "strided slice begin")?;
    validate_control_shape(end_shape, end.len(), "strided slice end")?;
    validate_control_shape(stride_shape, stride.len(), "strided slice stride")?;
    let rank = input.rank();
    if rank > 31 {
        return model("MediaPipe strided slice rank exceeds mask width");
    }
    if begin.len() != rank || end.len() != rank || stride.len() != rank {
        return model("MediaPipe strided slice controls must match input rank");
    }
    if begin_mask < 0 || end_mask < 0 || shrink_axis_mask < 0 {
        return model("MediaPipe strided slice mask is negative");
    }
    let valid_mask = if rank == 31 { i32::MAX } else { (1i32 << rank).saturating_sub(1) };
    if (begin_mask | end_mask | shrink_axis_mask) & !valid_mask != 0 {
        return model("MediaPipe strided slice mask references an out-of-range axis");
    }
    let mut output = input.clone();
    // Remove shrunk dimensions from high to low so original lower axes remain stable.
    for axis in (0..rank).rev() {
        let step = usize::try_from(stride[axis]).map_err(|_| Error::Model("MediaPipe strided slice only supports positive strides".into()))?;
        if step == 0 {
            return model("MediaPipe strided slice stride is zero");
        }
        let size = output.dims().get(axis).copied().ok_or_else(|| Error::Model("MediaPipe strided slice axis is out of range".into()))?;
        let shrink = (shrink_axis_mask
            & (1i32.checked_shl(u32::try_from(axis).map_err(|_| Error::Model("MediaPipe strided slice axis overflow".into()))?).unwrap_or(0)))
            != 0;
        let start = slice_bound(begin[axis], (begin_mask & bit(axis)) != 0, size, "begin")?;
        let finish = slice_bound(end[axis], (end_mask & bit(axis)) != 0, size, "end")?;
        if shrink {
            if step != 1 || start >= size {
                return model("MediaPipe strided slice shrink index is out of bounds or stepped");
            }
            let index = Tensor::from_vec(
                vec![u32::try_from(start).map_err(|_| Error::Model("MediaPipe strided slice index overflow".into()))?],
                1,
                output.device(),
            )?;
            output = output.index_select(&index, axis)?.squeeze(axis)?;
        } else {
            if finish < start {
                return model("MediaPipe strided slice has empty positive range");
            }
            let length = finish - start;
            if step == 1 {
                output = output.narrow(axis, start, length)?;
            } else {
                let first = u32::try_from(start).map_err(|_| Error::Model("MediaPipe strided slice index overflow".into()))?;
                let last = u32::try_from(finish).map_err(|_| Error::Model("MediaPipe strided slice bound overflow".into()))?;
                let step = u32::try_from(step).map_err(|_| Error::Model("MediaPipe strided slice stride overflow".into()))?;
                let index = Tensor::arange_step(first, last, step, output.device())?;
                output = output.index_select(&index, axis)?;
            }
        }
    }
    Ok(output)
}

fn slice_bound(value: i32, masked: bool, size: usize, name: &str) -> Result<usize> {
    if masked {
        return Ok(if name == "begin" { 0 } else { size });
    }
    let size_i32 = i32::try_from(size).map_err(|_| Error::Model("MediaPipe strided slice dimension overflows Int32".into()))?;
    let value = if value < 0 { value.checked_add(size_i32).unwrap_or(i32::MIN) } else { value };
    Ok(value.clamp(0, size_i32) as usize)
}

fn bit(axis: usize) -> i32 {
    u32::try_from(axis).ok().and_then(|axis| 1i32.checked_shl(axis)).unwrap_or(0)
}

fn normalize_axis(axis: i32, rank: usize, name: &str) -> Result<usize> {
    let rank = i32::try_from(rank).map_err(|_| Error::Model(format!("MediaPipe {name} rank overflows Int32")))?;
    let normalized =
        if axis < 0 { axis.checked_add(rank) } else { Some(axis) }.ok_or_else(|| Error::Model(format!("MediaPipe {name} axis overflows")))?;
    usize::try_from(normalized)
        .ok()
        .filter(|axis| *axis < usize::try_from(rank).unwrap_or(0))
        .ok_or_else(|| Error::Model(format!("MediaPipe {name} axis is out of range")))
}

fn normalize_axes(axes: &[i32], rank: usize, name: &str) -> Result<Vec<usize>> {
    // TFLite's Reduce kernels preserve input shape for an empty axis tensor.
    let mut normalized = Vec::with_capacity(axes.len());
    for &axis in axes {
        let axis = normalize_axis(axis, rank, name)?;
        if !normalized.contains(&axis) {
            normalized.push(axis);
        }
    }
    normalized.sort_unstable();
    Ok(normalized)
}

fn validate_control_shape(shape: &[usize], elements: usize, name: &str) -> Result<()> {
    if shape.len() > MAX_CONTROL_RANK || shape.iter().try_fold(1usize, |count, dim| count.checked_mul(*dim)) != Some(elements) {
        return model(format!("MediaPipe {name} has malformed Int32 control shape"));
    }
    Ok(())
}

fn finish(tensor: Tensor, expected: &TensorSpec) -> Result<Value> {
    if expected.dtype != crate::mediapipe_graph::StorageType::Float32 {
        return model("MediaPipe op output spec is not Float32");
    }
    if tensor.dtype() != DType::F32 || tensor.dims() != expected.shape {
        return model(format!(
            "MediaPipe op output shape/type mismatch: got {:?}/{:?}, expected {:?}/Float32",
            tensor.dims(),
            tensor.dtype(),
            expected.shape
        ));
    }
    Ok(Value::Float(tensor))
}

fn finish_value(value: Value, expected: &TensorSpec) -> Result<Value> {
    let tensor = match value {
        Value::Float(tensor) => tensor,
        Value::Int { .. } => return model("MediaPipe convolution returned an Int32 tensor"),
    };
    finish(tensor, expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;

    fn spec(shape: &'static [usize]) -> TensorSpec {
        TensorSpec { name: "test", shape, dtype: crate::mediapipe_graph::StorageType::Float32, offset: None, byte_len: 0 }
    }

    fn float(values: &[f32], shape: &'static [usize]) -> Value {
        Value::Float(Tensor::from_vec(values.to_vec(), shape, &Device::Cpu).expect("test tensor"))
    }

    fn ints(values: &[i32], shape: Vec<usize>) -> Value {
        Value::Int { values: values.to_vec(), shape }
    }

    #[test]
    fn binary_broadcasts_trailing_dimensions() {
        let value =
            execute(&Op::Add { activation: Activation::None }, &[float(&[1., 2., 3., 4.], &[2, 2]), float(&[10., 20.], &[2])], &spec(&[2, 2]))
                .expect("broadcast");
        let Value::Float(value) = value else { panic!("float output") };
        assert_eq!(value.to_vec2::<f32>().expect("values"), vec![vec![11., 22.], vec![13., 24.]]);
    }

    #[test]
    fn reshape_resolves_inferred_dimension_from_control() {
        let value =
            execute(&Op::Reshape { new_shape: &[] }, &[float(&[1., 2., 3., 4.], &[4]), ints(&[2, -1], vec![2])], &spec(&[2, 2])).expect("reshape");
        let Value::Float(value) = value else { panic!("float output") };
        assert_eq!(value.dims(), &[2, 2]);
    }

    #[test]
    fn reductions_normalize_and_deduplicate_axes() {
        let value =
            execute(&Op::Mean { keep_dims: true }, &[float(&[1., 3., 5., 7.], &[2, 2]), ints(&[-1, 1], vec![2])], &spec(&[2, 1])).expect("mean");
        let Value::Float(value) = value else { panic!("float output") };
        assert_eq!(value.to_vec2::<f32>().expect("values"), vec![vec![2.], vec![6.]]);
    }

    #[test]
    fn malformed_transpose_permutation_is_rejected() {
        let result = execute(&Op::Transpose, &[float(&[1., 2., 3., 4.], &[2, 2]), ints(&[0, 0], vec![2])], &spec(&[2, 2]));
        assert!(result.is_err());
    }

    #[test]
    fn strided_slice_applies_masks_and_positive_stride() {
        let value = execute(
            &Op::StridedSlice { begin_mask: 1, end_mask: 0, ellipsis_mask: 0, new_axis_mask: 0, shrink_axis_mask: 0 },
            &[float(&[0., 1., 2., 3., 4.], &[5]), ints(&[0], vec![1]), ints(&[5], vec![1]), ints(&[2], vec![1])],
            &spec(&[3]),
        )
        .expect("slice");
        let Value::Float(value) = value else { panic!("float output") };
        assert_eq!(value.to_vec1::<f32>().expect("values"), vec![0., 2., 4.]);
    }
}
