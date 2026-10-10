//! Exact MediaPipe convolution & pooling kernels on Candle.
//!
//! MediaPipe tensors are NHWC, while Candle's convolution and pooling kernels use NCHW.
//! TFLite convolution filters are OHWI; depthwise filters are `[1, kh, kw, cout]`.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{DType, Tensor};

use crate::mediapipe_graph::{Activation, Op, Padding, Value};

const MAX_ELEMENTS: usize = 16_000_000;
const MAX_BACKEND_SCRATCH_ELEMENTS: usize = 64 * 1024 * 1024;

/// Execute one convolution or pooling node.
pub(crate) fn execute(op: &Op, inputs: &[Value]) -> crate::Result<Value> {
    match *op {
        Op::Conv2d { stride_h, stride_w, dilation_h, dilation_w, padding, activation } => {
            execute_conv(inputs, stride_h, stride_w, dilation_h, dilation_w, padding, activation, false, 1)
        }
        Op::DepthwiseConv2d { stride_h, stride_w, dilation_h, dilation_w, depth_multiplier, padding, activation } => {
            execute_conv(inputs, stride_h, stride_w, dilation_h, dilation_w, padding, activation, true, depth_multiplier)
        }
        Op::MaxPool2d { stride_h, stride_w, filter_h, filter_w, padding, activation } => {
            execute_pool(inputs, stride_h, stride_w, filter_h, filter_w, padding, activation)
        }
        _ => Err(crate::Error::Model(format!("MediaPipe convolution module cannot execute {op:?}"))),
    }
}

fn execute_conv(
    inputs: &[Value],
    stride_h: usize,
    stride_w: usize,
    dilation_h: usize,
    dilation_w: usize,
    padding: Padding,
    activation: Activation,
    depthwise: bool,
    depth_multiplier: usize,
) -> crate::Result<Value> {
    if inputs.len() != 3 {
        return Err(crate::Error::Model(format!("MediaPipe convolution expects three inputs, got {}", inputs.len())));
    }
    if stride_h == 0 || stride_w == 0 || dilation_h == 0 || dilation_w == 0 {
        return Err(crate::Error::Model("MediaPipe convolution has a zero stride or dilation".into()));
    }
    // Candle 0.9.2 exposes scalar stride and dilation for conv2d.  Reject unequal values so
    // malformed graphs cannot silently receive an approximation using one axis's parameters.
    if stride_h != stride_w || dilation_h != dilation_w {
        return Err(crate::Error::Model("MediaPipe convolution requires equal H/W stride and dilation for Candle 0.9.2".into()));
    }
    if depthwise && depth_multiplier == 0 {
        return Err(crate::Error::Model("MediaPipe depthwise convolution has zero depth multiplier".into()));
    }

    let input = float_tensor(&inputs[0], "input")?;
    let filter = float_tensor(&inputs[1], "filter")?;
    let bias = float_tensor(&inputs[2], "bias")?;
    let [batch, height, width, channels] = shape4(input, "input")?;
    let filter_shape = shape4(filter, "filter")?;
    let bias_shape = bias.dims();
    if bias_shape.len() != 1 {
        return Err(crate::Error::Model(format!("MediaPipe convolution bias must be rank 1, got {:?}", bias_shape)));
    }
    let (out_channels, kernel_h, kernel_w) = if depthwise {
        if filter_shape[0] != 1 {
            return Err(crate::Error::Model("MediaPipe depthwise filter first dimension must be 1".into()));
        }
        let expected =
            channels.checked_mul(depth_multiplier).ok_or_else(|| crate::Error::Model("MediaPipe depthwise output channel count overflow".into()))?;
        if filter_shape[3] != expected {
            return Err(crate::Error::Model("MediaPipe depthwise filter has invalid output channels for depth multiplier".into()));
        }
        (filter_shape[3], filter_shape[1], filter_shape[2])
    } else {
        if filter_shape[3] != channels {
            return Err(crate::Error::Model(format!(
                "MediaPipe convolution filter input channels {} do not match input channels {channels}",
                filter_shape[3]
            )));
        }
        (filter_shape[0], filter_shape[1], filter_shape[2])
    };
    if out_channels == 0 || kernel_h == 0 || kernel_w == 0 || bias_shape[0] != out_channels {
        return Err(crate::Error::Model("MediaPipe convolution has invalid filter or bias shape".into()));
    }
    let elements = checked_elements(&[batch, height, width, channels], "input")?;
    if checked_elements(&filter_shape, "filter")? > MAX_ELEMENTS || checked_elements(bias_shape, "bias")? > MAX_ELEMENTS {
        return Err(crate::Error::Model("MediaPipe convolution parameter allocation exceeds element bound".into()));
    }
    if elements > MAX_ELEMENTS {
        return Err(crate::Error::Model("MediaPipe convolution input exceeds element bound".into()));
    }
    let effective_h = effective_kernel(kernel_h, dilation_h)?;
    let effective_w = effective_kernel(kernel_w, dilation_w)?;
    let (out_h, pad_top, pad_bottom) = convolution_axis(height, effective_h, stride_h, padding)?;
    let (out_w, pad_left, pad_right) = convolution_axis(width, effective_w, stride_w, padding)?;
    let padded_h = height
        .checked_add(pad_top)
        .and_then(|value| value.checked_add(pad_bottom))
        .ok_or_else(|| crate::Error::Model("MediaPipe convolution padded height overflow".into()))?;
    let padded_w = width
        .checked_add(pad_left)
        .and_then(|value| value.checked_add(pad_right))
        .ok_or_else(|| crate::Error::Model("MediaPipe convolution padded width overflow".into()))?;
    if checked_elements(&[batch, padded_h, padded_w, channels], "padded input")? > MAX_ELEMENTS
        || checked_elements(&[batch, out_h, out_w, out_channels], "output")? > MAX_ELEMENTS
    {
        return Err(crate::Error::Model("MediaPipe convolution allocation exceeds element bound".into()));
    }
    check_backend_scratch(batch, out_h, out_w, channels, kernel_h, kernel_w, if depthwise { channels } else { 1 })?;

    // Candle CPU can consume strided layouts, but Candle Metal 0.9.2's conv2d path is
    // safest with contiguous storage (its strided-kernel fallback has a broken copy path).
    let mut input_nchw = input.permute((0, 3, 1, 2))?.contiguous()?;
    if matches!(padding, Padding::Same) {
        input_nchw = input_nchw.pad_with_zeros(2, pad_top, pad_bottom)?.pad_with_zeros(3, pad_left, pad_right)?;
    }
    let kernel_oihw = if depthwise {
        // `[1, kh, kw, cout]` -> `[cout, 1, kh, kw]`.
        filter.permute((3, 0, 1, 2))?.contiguous()?
    } else {
        // `[cout, kh, kw, cin]` -> `[cout, cin, kh, kw]`.
        filter.permute((0, 3, 1, 2))?.contiguous()?
    };
    let groups = if depthwise { channels } else { 1 };
    let output_nchw = input_nchw.conv2d(&kernel_oihw, 0, stride_h, dilation_h, groups)?;
    if output_nchw.dims() != [batch, out_channels, out_h, out_w] {
        return Err(crate::Error::Model(format!("MediaPipe convolution produced unexpected NCHW shape {:?}", output_nchw.dims())));
    }
    let bias = bias.reshape((1, out_channels, 1, 1))?;
    let output_nchw = output_nchw.broadcast_add(&bias)?;
    let output_nchw = crate::mediapipe_ops::activate(&output_nchw, activation)?;
    let output = output_nchw.permute((0, 2, 3, 1))?;
    if output.dims() != [batch, out_h, out_w, out_channels] {
        return Err(crate::Error::Model(format!("MediaPipe convolution produced unexpected NHWC shape {:?}", output.dims())));
    }
    Ok(Value::Float(output))
}

fn execute_pool(
    inputs: &[Value],
    stride_h: usize,
    stride_w: usize,
    filter_h: usize,
    filter_w: usize,
    padding: Padding,
    activation: Activation,
) -> crate::Result<Value> {
    if inputs.len() != 1 {
        return Err(crate::Error::Model(format!("MediaPipe max pool expects one input, got {}", inputs.len())));
    }
    if stride_h == 0 || stride_w == 0 || filter_h == 0 || filter_w == 0 {
        return Err(crate::Error::Model("MediaPipe max pool has a zero stride or filter".into()));
    }
    let input = float_tensor(&inputs[0], "input")?;
    let [batch, height, width, channels] = shape4(input, "input")?;
    if checked_elements(&[batch, height, width, channels], "pool input")? > MAX_ELEMENTS {
        return Err(crate::Error::Model("MediaPipe max pool input exceeds element bound".into()));
    }
    let (out_h, pad_top, pad_bottom) = pool_axis(height, filter_h, stride_h, padding)?;
    let (out_w, pad_left, pad_right) = pool_axis(width, filter_w, stride_w, padding)?;
    if checked_elements(&[batch, out_h, out_w, channels], "pool output")? > MAX_ELEMENTS {
        return Err(crate::Error::Model("MediaPipe max pool output exceeds element bound".into()));
    }
    let mut input_nchw = input.permute((0, 3, 1, 2))?.contiguous()?;
    if matches!(padding, Padding::Same) {
        let padded_h = height
            .checked_add(pad_top)
            .and_then(|value| value.checked_add(pad_bottom))
            .ok_or_else(|| crate::Error::Model("MediaPipe max pool padded height overflow".into()))?;
        let padded_w = width
            .checked_add(pad_left)
            .and_then(|value| value.checked_add(pad_right))
            .ok_or_else(|| crate::Error::Model("MediaPipe max pool padded width overflow".into()))?;
        if checked_elements(&[batch, padded_h, padded_w, channels], "padded pool input")? > MAX_ELEMENTS {
            return Err(crate::Error::Model("MediaPipe max pool allocation exceeds element bound".into()));
        }
        input_nchw = pad_with_constant(input_nchw, 2, pad_top, pad_bottom, f32::NEG_INFINITY)?;
        input_nchw = pad_with_constant(input_nchw, 3, pad_left, pad_right, f32::NEG_INFINITY)?;
    }
    let output_nchw = input_nchw.max_pool2d_with_stride((filter_h, filter_w), (stride_h, stride_w))?;
    if output_nchw.dims() != [batch, channels, out_h, out_w] {
        return Err(crate::Error::Model(format!("MediaPipe max pool produced unexpected NCHW shape {:?}", output_nchw.dims())));
    }
    let output_nchw = crate::mediapipe_ops::activate(&output_nchw, activation)?;
    let output = output_nchw.permute((0, 2, 3, 1))?;
    if output.dims() != [batch, out_h, out_w, channels] {
        return Err(crate::Error::Model(format!("MediaPipe max pool produced unexpected NHWC shape {:?}", output.dims())));
    }
    Ok(Value::Float(output))
}

fn float_tensor<'a>(value: &'a Value, role: &str) -> crate::Result<&'a Tensor> {
    let tensor = value.float()?;
    if tensor.dtype() != DType::F32 {
        return Err(crate::Error::Model(format!("MediaPipe {role} tensor must be Float32, got {:?}", tensor.dtype())));
    }
    Ok(tensor)
}

fn shape4(tensor: &Tensor, role: &str) -> crate::Result<[usize; 4]> {
    let dims = tensor.dims();
    if dims.len() != 4 || dims.contains(&0) {
        return Err(crate::Error::Model(format!("MediaPipe {role} tensor must be non-empty rank 4, got {dims:?}")));
    }
    Ok([dims[0], dims[1], dims[2], dims[3]])
}

fn checked_elements(shape: &[usize], role: &str) -> crate::Result<usize> {
    shape.iter().try_fold(1usize, |count, dimension| {
        count.checked_mul(*dimension).ok_or_else(|| crate::Error::Model(format!("MediaPipe {role} element count overflow")))
    })
}

fn effective_kernel(kernel: usize, dilation: usize) -> crate::Result<usize> {
    kernel
        .checked_sub(1)
        .and_then(|value| value.checked_mul(dilation))
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| crate::Error::Model("MediaPipe effective kernel size overflow".into()))
}

fn convolution_axis(input: usize, effective: usize, stride: usize, padding: Padding) -> crate::Result<(usize, usize, usize)> {
    if input == 0 || effective == 0 || stride == 0 {
        return Err(crate::Error::Model("MediaPipe convolution axis has zero parameter".into()));
    }
    match padding {
        Padding::Valid => {
            if input < effective {
                return Err(crate::Error::Model("MediaPipe VALID convolution filter exceeds input".into()));
            }
            Ok(((input - effective) / stride + 1, 0, 0))
        }
        Padding::Same => {
            let output =
                input.checked_add(stride - 1).ok_or_else(|| crate::Error::Model("MediaPipe SAME convolution output overflow".into()))? / stride;
            let required = output
                .checked_sub(1)
                .and_then(|value| value.checked_mul(stride))
                .and_then(|value| value.checked_add(effective))
                .ok_or_else(|| crate::Error::Model("MediaPipe SAME convolution padding overflow".into()))?;
            let total = required.saturating_sub(input);
            Ok((output, total / 2, total - total / 2))
        }
    }
}

fn pool_axis(input: usize, filter: usize, stride: usize, padding: Padding) -> crate::Result<(usize, usize, usize)> {
    convolution_axis(input, filter, stride, padding)
}

fn pad_with_constant(tensor: Tensor, dim: usize, left: usize, right: usize, value: f32) -> crate::Result<Tensor> {
    if left == 0 && right == 0 {
        return Ok(tensor);
    }
    let dims = tensor.dims();
    let Some(_) = dims.get(dim) else {
        return Err(crate::Error::Model("MediaPipe padding dimension is out of range".into()));
    };
    let mut left_shape = dims.to_vec();
    left_shape[dim] = left;
    let mut right_shape = dims.to_vec();
    right_shape[dim] = right;
    let left_tensor = (left > 0).then(|| Tensor::full(value, left_shape, tensor.device())).transpose()?;
    let right_tensor = (right > 0).then(|| Tensor::full(value, right_shape, tensor.device())).transpose()?;
    let mut parts = Vec::with_capacity(3);
    if let Some(left_tensor) = left_tensor.as_ref() {
        parts.push(left_tensor);
    }
    parts.push(&tensor);
    if let Some(right_tensor) = right_tensor.as_ref() {
        parts.push(right_tensor);
    }
    Ok(Tensor::cat(&parts, dim)?)
}

fn check_backend_scratch(
    batch: usize,
    out_h: usize,
    out_w: usize,
    channels: usize,
    kernel_h: usize,
    kernel_w: usize,
    groups: usize,
) -> crate::Result<()> {
    let metal = checked_product(&[batch, out_h, out_w, channels, kernel_h, kernel_w], "Metal im2col scratch")?;
    let channels_per_group =
        channels.checked_div(groups).ok_or_else(|| crate::Error::Model("MediaPipe convolution scratch group count is invalid".into()))?;
    let cpu = checked_product(&[channels_per_group, kernel_h, kernel_w, 512], "CPU tiled convolution scratch")?;
    if metal > MAX_BACKEND_SCRATCH_ELEMENTS || cpu > MAX_BACKEND_SCRATCH_ELEMENTS {
        return Err(crate::Error::Model("MediaPipe convolution backend scratch allocation exceeds element bound".into()));
    }
    Ok(())
}

fn checked_product(values: &[usize], role: &str) -> crate::Result<usize> {
    values
        .iter()
        .try_fold(1usize, |total, value| total.checked_mul(*value).ok_or_else(|| crate::Error::Model(format!("{role} element count overflow"))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;

    #[test]
    fn same_padding_is_asymmetric_when_required() -> crate::Result<()> {
        assert_eq!(convolution_axis(5, 4, 2, Padding::Same)?, (3, 1, 2));
        Ok(())
    }

    #[test]
    fn max_pool_same_uses_negative_infinity_for_negative_inputs() -> crate::Result<()> {
        let input = Tensor::new(&[[[[-1.0f32], [-2.0]], [[-4.0], [-5.0]]]], &Device::Cpu)?;
        let value = execute(
            &Op::MaxPool2d { stride_h: 2, stride_w: 2, filter_h: 2, filter_w: 2, padding: Padding::Same, activation: Activation::None },
            &[Value::Float(input)],
        )?;
        let Value::Float(output) = value else { return Err(crate::Error::Model("pool test returned integer".into())) };
        assert_eq!(output.to_vec4::<f32>()?, vec![vec![vec![vec![-1.0]]]]);
        Ok(())
    }

    #[test]
    fn depthwise_filter_layout_expands_output_channels() -> crate::Result<()> {
        let input = Tensor::new(&[[[[2.0f32], [4.0]]]], &Device::Cpu)?;
        let filter = Tensor::new(&[[[[2.0f32, 3.0]]]], &Device::Cpu)?;
        let bias = Tensor::zeros(2, DType::F32, &Device::Cpu)?;
        let value = execute(
            &Op::DepthwiseConv2d {
                stride_h: 1,
                stride_w: 1,
                dilation_h: 1,
                dilation_w: 1,
                depth_multiplier: 2,
                padding: Padding::Valid,
                activation: Activation::None,
            },
            &[Value::Float(input), Value::Float(filter), Value::Float(bias)],
        )?;
        let Value::Float(output) = value else { return Err(crate::Error::Model("depthwise test returned integer".into())) };
        assert_eq!(output.to_vec4::<f32>()?, vec![vec![vec![vec![4.0, 6.0], vec![8.0, 12.0]]]]);
        Ok(())
    }

    #[test]
    fn fused_activation_is_applied_after_bias() -> crate::Result<()> {
        let input = Tensor::new(&[[[[-2.0f32], [2.0]], [[8.0], [1.0]]]], &Device::Cpu)?;
        let filter = Tensor::new(&[[[[1.0f32]]]], &Device::Cpu)?;
        let bias = Tensor::zeros(1, DType::F32, &Device::Cpu)?;
        let value = execute(
            &Op::Conv2d { stride_h: 1, stride_w: 1, dilation_h: 1, dilation_w: 1, padding: Padding::Valid, activation: Activation::Relu6 },
            &[Value::Float(input), Value::Float(filter), Value::Float(bias)],
        )?;
        let Value::Float(output) = value else { return Err(crate::Error::Model("activation test returned integer".into())) };
        assert_eq!(output.to_vec4::<f32>()?, vec![vec![vec![vec![0.0], vec![2.0]], vec![vec![6.0], vec![1.0]]]]);
        Ok(())
    }
}
