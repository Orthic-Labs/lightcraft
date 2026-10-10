//! Deterministic low-level spatial evidence for offline Personal Auto evaluation.
//!
//! This extractor records layout-conditioned photometric statistics only. It does not identify
//! subjects, faces, skies, scenes, or photographic intent, and it is never part of production
//! `auto_tone`.

use lightcraft_color::luminance_2020;
use lightcraft_color::perceptual::oklab_from_2020;
use lightcraft_develop::DevelopSettings;
use lightcraft_raster::Rgb32f;
use serde::{Deserialize, Serialize};

use crate::SourceInfo;
use crate::personal_auto::{MAX_FEATURE_PIXELS, PersonalAutoError};

/// Versioned spatial feature contract. Values are ordered row-major by 2×2 cell.
pub const SPATIAL_FEATURE_SCHEMA: &str = "lightcraft.personal-auto-spatial-features.v1";
/// Number of fixed statistics emitted per cell.
pub const SPATIAL_FEATURES_PER_CELL: usize = 3;
/// Fixed spatial grid width and height.
pub const SPATIAL_GRID_WIDTH: usize = 2;
pub const SPATIAL_GRID_HEIGHT: usize = 2;
/// Number of spatial features in one vector.
pub const SPATIAL_FEATURE_COUNT: usize = SPATIAL_GRID_WIDTH * SPATIAL_GRID_HEIGHT * SPATIAL_FEATURES_PER_CELL;
/// EV clamp applied before spatial aggregation.
pub const SPATIAL_EV_LIMIT: f64 = 32.0;
/// Oklab chroma clamp applied before spatial aggregation.
pub const SPATIAL_CHROMA_LIMIT: f64 = 4.0;

/// Stable row-major names: each cell emits EV mean, EV population standard deviation, and mean
/// Oklab chroma. Cell coordinates use zero-based `(row, column)` with origin at the top left.
pub const SPATIAL_FEATURE_NAMES: [&str; SPATIAL_FEATURE_COUNT] = [
    "spatial.g00.ev.mean",
    "spatial.g00.ev.stddev",
    "spatial.g00.chroma.mean",
    "spatial.g01.ev.mean",
    "spatial.g01.ev.stddev",
    "spatial.g01.chroma.mean",
    "spatial.g10.ev.mean",
    "spatial.g10.ev.stddev",
    "spatial.g10.chroma.mean",
    "spatial.g11.ev.mean",
    "spatial.g11.ev.stddev",
    "spatial.g11.chroma.mean",
];

/// Fixed-size spatial evidence vector supplied by the offline evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpatialFeatureVector(pub [f64; SPATIAL_FEATURE_COUNT]);

impl SpatialFeatureVector {
    /// Construct a vector after rejecting non-finite values.
    pub fn new(values: [f64; SPATIAL_FEATURE_COUNT]) -> Result<Self, PersonalAutoError> {
        let out = Self(values);
        out.validate()?;
        Ok(out)
    }

    /// Validate fixed dimensionality and finite values.
    pub fn validate(&self) -> Result<(), PersonalAutoError> {
        if self.0.iter().all(|value| value.is_finite()) { Ok(()) } else { Err(PersonalAutoError::NonFinite("spatial feature vector".into())) }
    }

    /// Read one feature by fixed index.
    pub fn get(&self, index: usize) -> Option<f64> {
        self.0.get(index).copied()
    }

    /// Extract spatial evidence from the same upright 512px Box proxy and WB-only preparation
    /// used by the scalar Personal Auto features. Exposure and all tone controls are ignored.
    pub fn from_pipeline(src: &Rgb32f, info: &SourceInfo, baseline: &DevelopSettings) -> Result<Self, PersonalAutoError> {
        let pixel_count = src.width.checked_mul(src.height).ok_or_else(|| PersonalAutoError::Malformed("source dimensions overflow".into()))?;
        if pixel_count > MAX_FEATURE_PIXELS {
            return Err(PersonalAutoError::Malformed("source image exceeds feature pixel cap".into()));
        }
        crate::cull::validate_image(src).map_err(|error| match error {
            crate::cull::MeasurementError::NonFinitePixel { .. } => PersonalAutoError::NonFinite("source image".into()),
            _ => PersonalAutoError::Malformed(format!("source image: {error}")),
        })?;
        if !baseline.wb.temp.is_finite() || !baseline.wb.tint.is_finite() || !info.as_shot_temp.is_finite() || !info.as_shot_tint.is_finite() {
            return Err(PersonalAutoError::NonFinite("spatial source settings".into()));
        }

        let mut img = lightcraft_raster::resample::fit(src, 512, 512, lightcraft_raster::resample::Filter::Box);
        let wb_only = DevelopSettings { wb: baseline.wb, ..DevelopSettings::default() };
        crate::local::scene_linear_pre(&mut img, info, &wb_only);

        let mut count = [0usize; SPATIAL_GRID_WIDTH * SPATIAL_GRID_HEIGHT];
        let mut sum_ev = [0.0f64; SPATIAL_GRID_WIDTH * SPATIAL_GRID_HEIGHT];
        let mut sum_ev_squared = [0.0f64; SPATIAL_GRID_WIDTH * SPATIAL_GRID_HEIGHT];
        let mut sum_chroma = [0.0f64; SPATIAL_GRID_WIDTH * SPATIAL_GRID_HEIGHT];
        for y in 0..img.height {
            for x in 0..img.width {
                let index = y
                    .checked_mul(img.width)
                    .and_then(|row| row.checked_add(x))
                    .ok_or_else(|| PersonalAutoError::Malformed("spatial proxy index overflow".into()))?;
                let pixel = img.data.get(index).copied().ok_or_else(|| PersonalAutoError::Malformed("spatial proxy storage mismatch".into()))?;
                if pixel.iter().any(|value| !value.is_finite()) {
                    return Err(PersonalAutoError::NonFinite("spatial WB proxy".into()));
                }
                let y_luma = f64::from(luminance_2020(pixel));
                if !y_luma.is_finite() {
                    return Err(PersonalAutoError::NonFinite("spatial luminance".into()));
                }
                if y_luma <= 1e-6 {
                    continue;
                }
                let ev = (y_luma / 0.18).log2();
                if !ev.is_finite() {
                    return Err(PersonalAutoError::NonFinite("spatial EV".into()));
                }
                let lab = oklab_from_2020(pixel);
                let chroma = f64::from(lab[1]).hypot(f64::from(lab[2]));
                if !chroma.is_finite() {
                    return Err(PersonalAutoError::NonFinite("spatial chroma".into()));
                }
                let cell_x =
                    x.checked_mul(SPATIAL_GRID_WIDTH).ok_or_else(|| PersonalAutoError::Malformed("spatial cell index overflow".into()))? / img.width;
                let cell_y = y.checked_mul(SPATIAL_GRID_HEIGHT).ok_or_else(|| PersonalAutoError::Malformed("spatial cell index overflow".into()))?
                    / img.height;
                let cell = cell_y
                    .checked_mul(SPATIAL_GRID_WIDTH)
                    .and_then(|row| row.checked_add(cell_x))
                    .filter(|cell| *cell < count.len())
                    .ok_or_else(|| PersonalAutoError::Malformed("spatial cell index out of bounds".into()))?;
                let ev = ev.clamp(-SPATIAL_EV_LIMIT, SPATIAL_EV_LIMIT);
                let chroma = chroma.clamp(0.0, SPATIAL_CHROMA_LIMIT);
                let count_value = count.get_mut(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial count index out of bounds".into()))?;
                let sum_ev_value = sum_ev.get_mut(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial EV index out of bounds".into()))?;
                let sum_ev_squared_value =
                    sum_ev_squared.get_mut(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial EV variance index out of bounds".into()))?;
                let sum_chroma_value =
                    sum_chroma.get_mut(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial chroma index out of bounds".into()))?;
                *count_value = count_value.saturating_add(1);
                *sum_ev_value += ev;
                *sum_ev_squared_value += ev * ev;
                *sum_chroma_value += chroma;
            }
        }
        let valid_pixels = count
            .iter()
            .try_fold(0usize, |total, value| total.checked_add(*value))
            .ok_or_else(|| PersonalAutoError::Malformed("spatial population overflow".into()))?;
        if valid_pixels == 0 {
            return Err(PersonalAutoError::Malformed("source has no finite spatial scene evidence".into()));
        }

        // Empty cells have an explicit deterministic zero tuple. The global valid-pixel check
        // above keeps an all-black source from being silently accepted as a neutral layout.
        let mut values = [0.0; SPATIAL_FEATURE_COUNT];
        for cell in 0..count.len() {
            let n = *count.get(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial count index out of bounds".into()))?;
            if n == 0 {
                continue;
            }
            let n = n as f64;
            let sum_ev_value = *sum_ev.get(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial EV index out of bounds".into()))?;
            let sum_ev_squared_value =
                *sum_ev_squared.get(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial EV variance index out of bounds".into()))?;
            let sum_chroma_value = *sum_chroma.get(cell).ok_or_else(|| PersonalAutoError::Malformed("spatial chroma index out of bounds".into()))?;
            let mean_ev = sum_ev_value / n;
            let variance = (sum_ev_squared_value / n - mean_ev * mean_ev).max(0.0);
            let stddev_ev = variance.sqrt();
            let mean_chroma = sum_chroma_value / n;
            let offset = cell * SPATIAL_FEATURES_PER_CELL;
            let cell_values = values
                .get_mut(offset..offset + SPATIAL_FEATURES_PER_CELL)
                .ok_or_else(|| PersonalAutoError::Malformed("spatial feature slice out of bounds".into()))?;
            let ev_mean = cell_values.get_mut(0).ok_or_else(|| PersonalAutoError::Malformed("spatial EV mean slot missing".into()))?;
            *ev_mean = mean_ev;
            let ev_stddev = cell_values.get_mut(1).ok_or_else(|| PersonalAutoError::Malformed("spatial EV stddev slot missing".into()))?;
            *ev_stddev = stddev_ev;
            let chroma_mean = cell_values.get_mut(2).ok_or_else(|| PersonalAutoError::Malformed("spatial chroma mean slot missing".into()))?;
            *chroma_mean = mean_chroma;
        }
        Self::new(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> (SourceInfo, DevelopSettings) {
        (SourceInfo::default(), DevelopSettings::default())
    }

    fn quadrants(swapped: bool) -> Rgb32f {
        Rgb32f::from_fn(8, 8, |x, y| {
            let values = if swapped { [0.04, 0.16, 0.64, 0.25] } else { [0.04, 0.16, 0.25, 0.64] };
            let cell = (y / 4) * 2 + (x / 4);
            [values[cell]; 3]
        })
    }

    #[test]
    fn cell_order_encodes_quadrant_layout() {
        let (info, settings) = settings();
        let features = SpatialFeatureVector::from_pipeline(&quadrants(false), &info, &settings).unwrap();
        let expected = |value: f64| (value / 0.18).log2();
        assert!((features.0[0] - expected(0.04)).abs() < 1e-5);
        assert!((features.0[3] - expected(0.16)).abs() < 1e-5);
        assert!((features.0[6] - expected(0.25)).abs() < 1e-5);
        assert!((features.0[9] - expected(0.64)).abs() < 1e-5);
        assert!(features.0[1].abs() < 1e-5);
        assert!(features.0[2].abs() < 1e-5);

        let varied = Rgb32f::from_fn(8, 8, |x, y| {
            let bases = [0.04, 0.16, 0.25, 0.64];
            let base = bases[(y / 4) * 2 + (x / 4)];
            let value = if x % 2 == 0 { base } else { base * 2.0 };
            [value; 3]
        });
        let varied_features = SpatialFeatureVector::from_pipeline(&varied, &info, &settings).unwrap();
        let first = expected(0.04);
        let second = expected(0.08);
        let expected_mean = (first + second) / 2.0;
        let expected_stddev = (((first - expected_mean).powi(2) + (second - expected_mean).powi(2)) / 2.0).sqrt();
        assert!((varied_features.0[0] - expected_mean).abs() < 1e-5);
        assert!((varied_features.0[1] - expected_stddev).abs() < 1e-5);
    }

    #[test]
    fn spatial_permutation_changes_vector_and_repeat_is_stable() {
        let (info, settings) = settings();
        let first = SpatialFeatureVector::from_pipeline(&quadrants(false), &info, &settings).unwrap();
        let repeated = SpatialFeatureVector::from_pipeline(&quadrants(false), &info, &settings).unwrap();
        let swapped = SpatialFeatureVector::from_pipeline(&quadrants(true), &info, &settings).unwrap();
        assert_eq!(first, repeated);
        assert_ne!(first, swapped);
    }

    #[test]
    fn rejects_nonfinite_and_all_black_sources() {
        let (info, settings) = settings();
        let nonfinite = Rgb32f::from_fn(2, 2, |_, _| [f32::NAN; 3]);
        assert!(matches!(SpatialFeatureVector::from_pipeline(&nonfinite, &info, &settings), Err(PersonalAutoError::NonFinite(_))));
        let black = Rgb32f::from_fn(2, 2, |_, _| [0.0; 3]);
        assert!(matches!(SpatialFeatureVector::from_pipeline(&black, &info, &settings), Err(PersonalAutoError::Malformed(_))));
    }

    #[test]
    fn rejects_malformed_storage_dimensions_and_pixel_cap() {
        let (info, settings) = settings();
        let malformed = Rgb32f { width: 2, height: 2, data: vec![[0.18; 3]; 3] };
        assert!(matches!(SpatialFeatureVector::from_pipeline(&malformed, &info, &settings), Err(PersonalAutoError::Malformed(_))));
        let oversized = Rgb32f { width: MAX_FEATURE_PIXELS + 1, height: 1, data: Vec::new() };
        assert!(matches!(SpatialFeatureVector::from_pipeline(&oversized, &info, &settings), Err(PersonalAutoError::Malformed(_))));
        let overflowing = Rgb32f { width: usize::MAX, height: 2, data: Vec::new() };
        assert!(matches!(SpatialFeatureVector::from_pipeline(&overflowing, &info, &settings), Err(PersonalAutoError::Malformed(_))));
    }

    #[test]
    fn rejects_nonfinite_derived_wb_proxy() {
        let (info, mut settings) = settings();
        settings.wb.mode = lightcraft_develop::WbMode::Custom;
        settings.wb.temp = 2000.0;
        settings.wb.tint = 0.0;
        let matrix = crate::local::wb_matrix_for(&info, &settings).expect("custom WB matrix");
        assert!(matrix[0][0].is_finite() && matrix[0][0] > 1.0, "{matrix:?}");
        let source = Rgb32f::from_fn(2, 2, |_, _| [f32::MAX, 0.0, 0.0]);
        assert!(matches!(SpatialFeatureVector::from_pipeline(&source, &info, &settings), Err(PersonalAutoError::NonFinite(_))));
    }

    #[test]
    fn empty_cells_use_zero_tuple() {
        let (info, settings) = settings();
        let image = Rgb32f::from_fn(1, 1, |_, _| [0.18; 3]);
        let features = SpatialFeatureVector::from_pipeline(&image, &info, &settings).unwrap();
        assert!(features.0[0].abs() < 1e-6);
        assert!(features.0[3..].iter().all(|value| *value == 0.0));
    }
}
