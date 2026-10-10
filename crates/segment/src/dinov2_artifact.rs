//! Immutable, verified-byte loader for the pinned DINOv2 ViT-S/14 artifact.
//!
//! The inventory is generated from a static inspection of the official PyTorch zip container.
//! Loading uses only the already-verified byte ranges in that inventory; it does not parse zip or
//! pickle data, reopen the path, or execute pickle instructions.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;

use crate::dinov2::DinoV2;
use crate::dinov2_inventory::TensorSpec;
use crate::{Error, Result};

/// Exact byte length of the pinned DINOv2 ViT-S/14 artifact.
pub const MODEL_BYTES: usize = crate::dinov2_inventory::MODEL_BYTES;
/// SHA-256 of the exact pinned artifact bytes.
pub const MODEL_SHA256: &str = crate::dinov2_inventory::MODEL_SHA256;

const TENSOR_COUNT: usize = 175;

fn inventory() -> &'static [TensorSpec] {
    &crate::dinov2_inventory::TENSOR_SPECS
}

/// Load a pinned DINOv2 artifact from a path with one bounded read.
pub fn load_file(path: &Path, device: &Device) -> Result<DinoV2> {
    let file = File::open(path).map_err(|error| Error::Model(format!("{}: {error}", path.display())))?;
    let max_read = u64::try_from(MODEL_BYTES)
        .ok()
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or_else(|| Error::Model("DINOv2 artifact byte limit overflow".into()))?;
    let mut bytes = Vec::with_capacity(MODEL_BYTES.saturating_add(1));
    let mut bounded = file.take(max_read);
    bounded.read_to_end(&mut bytes).map_err(|error| Error::Model(format!("{}: {error}", path.display())))?;
    load_verified_bytes(bytes, device)
}

/// Load DINOv2 from owned bytes after verifying their exact pinned identity.
///
/// All descriptor, range, overlap, and finite-value checks complete before the first Candle
/// tensor is allocated. The input bytes are the only source of tensor data and are dropped after
/// the model has been built.
pub fn load_verified_bytes(bytes: Vec<u8>, device: &Device) -> Result<DinoV2> {
    verify_artifact_bytes(&bytes)?;
    validate_specs(&bytes)?;

    let mut tensors = std::collections::HashMap::with_capacity(inventory().len());
    for spec in inventory() {
        let end = spec.offset.checked_add(spec.byte_len).ok_or_else(|| Error::Model(format!("DINOv2 tensor {} range overflow", spec.name)))?;
        let raw = bytes.get(spec.offset..end).ok_or_else(|| Error::Model(format!("DINOv2 tensor {} range is outside artifact", spec.name)))?;
        let values = raw
            .chunks_exact(4)
            .map(|chunk| {
                let &[a, b, c, d] = chunk else {
                    // `chunks_exact(4)` is checked by descriptor validation. Keep this branch
                    // fallible so a future change cannot turn malformed bytes into a panic.
                    return Err(Error::Model(format!("DINOv2 tensor {} has an incomplete F32 value", spec.name)));
                };
                Ok(f32::from_le_bytes([a, b, c, d]))
            })
            .collect::<Result<Vec<_>>>()?;
        let tensor = Tensor::from_vec(values, spec.shape, device)?;
        tensors.insert(spec.name.to_owned(), tensor);
    }
    if tensors.len() != TENSOR_COUNT {
        return Err(Error::Model(format!("DINOv2 inventory produced {} tensors; expected {TENSOR_COUNT}", tensors.len())));
    }
    let vb = VarBuilder::from_tensors(tensors, DType::F32, device);
    DinoV2::from_var_builder(vb)
}

fn verify_artifact_bytes(bytes: &[u8]) -> Result<()> {
    let size = u64::try_from(MODEL_BYTES).map_err(|_| Error::Model("DINOv2 artifact byte limit overflow".into()))?;
    let spec = lightcraft_fetch::FileSpec { name: "dinov2_vits14_pretrain.pth", size: Some(size), sha256: Some(MODEL_SHA256), max: size };
    let verified = lightcraft_fetch::verify_bytes(&spec, bytes).map_err(Error::Model)?;
    if !verified {
        return Err(Error::Model(format!("DINOv2 artifact must be exactly {MODEL_BYTES} bytes with SHA-256 {MODEL_SHA256}")));
    }
    Ok(())
}

fn validate_specs(bytes: &[u8]) -> Result<()> {
    let specs = inventory();
    validate_spec_layout(specs, bytes.len())?;
    validate_finite_ranges(specs, bytes)
}

fn validate_finite_ranges(specs: &[TensorSpec], bytes: &[u8]) -> Result<()> {
    for spec in specs {
        let end = spec.offset.checked_add(spec.byte_len).ok_or_else(|| Error::Model(format!("DINOv2 tensor {} range overflow", spec.name)))?;
        let raw = bytes.get(spec.offset..end).ok_or_else(|| Error::Model(format!("DINOv2 tensor {} range is outside artifact", spec.name)))?;
        for chunk in raw.chunks_exact(4) {
            let &[a, b, c, d] = chunk else {
                return Err(Error::Model(format!("DINOv2 tensor {} has an incomplete F32 value", spec.name)));
            };
            if !f32::from_le_bytes([a, b, c, d]).is_finite() {
                return Err(Error::Model(format!("DINOv2 tensor {} contains a non-finite F32", spec.name)));
            }
        }
    }
    Ok(())
}

fn validate_spec_layout(specs: &[TensorSpec], artifact_len: usize) -> Result<()> {
    validate_spec_layout_inner(specs, artifact_len, true)
}

fn validate_spec_layout_inner(specs: &[TensorSpec], artifact_len: usize, enforce_count: bool) -> Result<()> {
    if enforce_count && specs.len() != TENSOR_COUNT {
        return Err(Error::Model(format!("DINOv2 inventory has {} tensors; expected {TENSOR_COUNT}", specs.len())));
    }
    let mut names = HashSet::with_capacity(specs.len());
    for spec in specs {
        if spec.name.is_empty() || !names.insert(spec.name) {
            return Err(Error::Model(format!("DINOv2 inventory has a duplicate or empty tensor name {:?}", spec.name)));
        }
        if spec.shape.is_empty() || spec.shape.iter().any(|dimension| *dimension == 0) {
            return Err(Error::Model(format!("DINOv2 tensor {} has an empty shape", spec.name)));
        }
        let elements = spec
            .shape
            .iter()
            .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
            .ok_or_else(|| Error::Model(format!("DINOv2 tensor {} shape element count overflows", spec.name)))?;
        let expected_bytes = elements
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| Error::Model(format!("DINOv2 tensor {} byte length overflows", spec.name)))?;
        if spec.byte_len != expected_bytes || spec.byte_len % std::mem::size_of::<f32>() != 0 {
            return Err(Error::Model(format!("DINOv2 tensor {} byte length does not match shape", spec.name)));
        }
        let end = spec.offset.checked_add(spec.byte_len).ok_or_else(|| Error::Model(format!("DINOv2 tensor {} range overflows", spec.name)))?;
        if end > artifact_len {
            return Err(Error::Model(format!("DINOv2 tensor {} range is outside artifact", spec.name)));
        }
    }
    for (index, first) in specs.iter().enumerate() {
        let first_end =
            first.offset.checked_add(first.byte_len).ok_or_else(|| Error::Model(format!("DINOv2 tensor {} range overflows", first.name)))?;
        for second in specs.iter().skip(index + 1) {
            let second_end =
                second.offset.checked_add(second.byte_len).ok_or_else(|| Error::Model(format!("DINOv2 tensor {} range overflows", second.name)))?;
            if first.offset < second_end && second.offset < first_end {
                return Err(Error::Model(format!("DINOv2 tensor ranges overlap: {} and {}", first.name, second.name)));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_length_before_inventory_or_tensor_work() {
        assert!(load_verified_bytes(Vec::new(), &Device::Cpu).is_err());
    }

    #[test]
    fn rejects_wrong_hash_without_official_weights() {
        let bytes = vec![0u8; MODEL_BYTES];
        assert!(verify_artifact_bytes(&bytes).is_err());
    }

    #[test]
    fn pinned_inventory_has_expected_nonoverlapping_layout() {
        assert!(validate_spec_layout(inventory(), MODEL_BYTES).is_ok());
    }

    #[test]
    fn inventory_validation_rejects_overlap_and_nonfinite_values() {
        const FIRST: TensorSpec = TensorSpec { name: "first", shape: &[1], offset: 0, byte_len: 4 };
        const SECOND: TensorSpec = TensorSpec { name: "second", shape: &[1], offset: 2, byte_len: 4 };
        let overlap = [FIRST, SECOND];
        assert!(validate_spec_layout_inner(&overlap, 8, false).is_err());

        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&f32::NAN.to_le_bytes());
        let finite_check = [FIRST];
        assert!(validate_specs_for_test(&finite_check, &bytes).is_err());
    }

    fn validate_specs_for_test(specs: &[TensorSpec], bytes: &[u8]) -> Result<()> {
        // Keep production's finite-value rule exercised without allocating the full artifact.
        if specs.len() != 1 {
            return Err(Error::Model("test inventory count".into()));
        }
        validate_spec_layout_inner(specs, bytes.len(), false)?;
        validate_finite_ranges(specs, bytes)
    }
}
