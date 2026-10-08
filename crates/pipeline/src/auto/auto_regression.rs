use super::{AutoTone, auto_tone, percentile};
use lightcraft_color::perceptual::oklab_from_2020;
use lightcraft_color::transfer::decode_srgb8;
use lightcraft_color::{REC2020, SRGB, luminance_2020};
use lightcraft_develop::DevelopSettings;
use lightcraft_raster::{Rgb32f, Rgba8};

use crate::{RenderRequest, SourceInfo};

fn finite_outputs(a: AutoTone) -> bool {
    [a.exposure, a.contrast, a.highlights, a.shadows, a.whites, a.blacks, a.vibrance, a.saturation].iter().all(|v| v.is_finite())
}

fn source_ev(src: &Rgb32f) -> Vec<f32> {
    let mut values: Vec<_> = src
        .data
        .iter()
        .filter_map(|c| {
            let y = luminance_2020(*c);
            if !y.is_finite() || y <= 1e-6 {
                return None;
            }
            let v = (y / 0.18).log2();
            v.is_finite().then_some(v)
        })
        .collect();
    values.sort_by(|a, b| a.total_cmp(b));
    values
}

fn median_after(src: &Rgb32f, a: AutoTone) -> f32 {
    let values = source_ev(src);
    percentile(&values, 0.5) + a.exposure as f32
}

fn low_key_scene() -> Rgb32f {
    Rgb32f::from_fn(64, 64, |x, y| if (x + y) % 17 == 0 { [0.04, 0.035, 0.03] } else { [0.01, 0.01, 0.012] })
}

fn dark_scene_with_skin_highlight() -> Rgb32f {
    Rgb32f::from_fn(64, 64, |x, y| if x < 13 && y < 13 { [0.35, 0.16, 0.09] } else { [0.01, 0.01, 0.012] })
}

fn ordinary_underexposed_scene() -> Rgb32f {
    Rgb32f::from_fn(64, 64, |x, _| {
        if x < 13 {
            [0.55, 0.48, 0.4]
        } else if x < 45 {
            [0.045, 0.04, 0.035]
        } else {
            [0.02, 0.018, 0.016]
        }
    })
}

fn high_key_scene() -> Rgb32f {
    Rgb32f::from_fn(64, 64, |x, y| if (x + y) % 11 == 0 { [0.65, 0.65, 0.62] } else { [1.1, 1.1, 1.05] })
}

fn backlit_scene() -> Rgb32f {
    Rgb32f::from_fn(64, 64, |x, _| if x < 52 { [0.012, 0.01, 0.009] } else { [4.0, 3.8, 3.4] })
}

fn saturated_scene() -> Rgb32f {
    Rgb32f::from_fn(64, 64, |x, y| if (x + y) % 2 == 0 { [0.02, 0.05, 0.8] } else { [0.7, 0.03, 0.02] })
}

fn neutral_scene() -> Rgb32f {
    Rgb32f::from_fn(64, 64, |x, y| {
        let v = 0.08 + (x + y) as f32 * 0.002;
        [v, v, v]
    })
}

fn render_with(src: &Rgb32f, settings: &DevelopSettings) -> Rgba8 {
    let info = SourceInfo::default();
    crate::render(src, &info, settings, &RenderRequest::fit(64, 64)).image
}

fn auto_settings(src: &Rgb32f) -> DevelopSettings {
    let info = SourceInfo::default();
    let mut settings = DevelopSettings::default();
    let auto = auto_tone(src, &info, &settings);
    settings.light.exposure = auto.exposure;
    settings.light.contrast = auto.contrast;
    settings.light.highlights = auto.highlights;
    settings.light.shadows = auto.shadows;
    settings.light.whites = auto.whites;
    settings.light.blacks = auto.blacks;
    settings.color.vibrance = auto.vibrance;
    settings.color.saturation = auto.saturation;
    settings
}

fn auto_render(src: &Rgb32f) -> Rgba8 {
    render_with(src, &auto_settings(src))
}

fn legacy_settings(src: &Rgb32f) -> DevelopSettings {
    let values = source_ev(src);
    let median = percentile(&values, 0.5);
    let exposure = (-median * 0.85 - 0.1).clamp(-4.0, 4.0);
    let p01 = percentile(&values, 0.01) + exposure;
    let p05 = percentile(&values, 0.05) + exposure;
    let p95 = percentile(&values, 0.95) + exposure;
    let p995 = percentile(&values, 0.995) + exposure;
    let mut settings = DevelopSettings::default();
    settings.light.exposure = exposure as f64;
    settings.light.contrast = ((6.5 - (p95 - p05)) * 6.0).clamp(-20.0, 30.0) as f64;
    settings.light.highlights = (if p995 > 2.2 { -((p995 - 2.2) * 38.0).min(90.0) } else { 0.0 }) as f64;
    settings.light.shadows = (if p05 < -4.0 { ((-4.0 - p05) * 22.0).min(70.0) } else { 0.0 }) as f64;
    settings.light.whites = (if p995 < 1.8 { ((1.8 - p995) * 25.0).min(40.0) } else { -((p995 - 3.5).max(0.0) * 10.0).min(30.0) }) as f64;
    settings.light.blacks = (if p01 > -5.0 { -((p01 + 5.0) * 10.0).min(35.0) } else { ((-7.0 - p01).max(0.0) * 8.0).min(20.0) }) as f64;
    settings.color.vibrance = 12.0;
    settings.color.saturation = 3.0;
    settings
}

fn output_luma(image: &Rgba8) -> Vec<f32> {
    image
        .data
        .iter()
        .map(|p| {
            let c = [decode_srgb8(p[0]), decode_srgb8(p[1]), decode_srgb8(p[2])];
            c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722
        })
        .collect()
}

fn output_chroma(image: &Rgba8) -> Vec<f32> {
    image
        .data
        .iter()
        .map(|p| {
            let encoded = [decode_srgb8(p[0]), decode_srgb8(p[1]), decode_srgb8(p[2])];
            let c = SRGB.to_space(&REC2020).apply_f32(encoded);
            let lab = oklab_from_2020(c);
            lab[1].hypot(lab[2])
        })
        .collect()
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

#[test]
fn low_key_auto_keeps_night_scene_dark_in_render() {
    let src = low_key_scene();
    let a = auto_tone(&src, &SourceInfo::default(), &DevelopSettings::default());
    let median = median_after(&src, a);
    let pixels = output_luma(&auto_render(&src));
    let legacy = output_luma(&render_with(&src, &legacy_settings(&src)));
    let lit = pixels.iter().filter(|v| **v > 0.65).count() as f32 / pixels.len() as f32;
    assert!(
        median < -1.0 && lit < 0.2 && mean(&pixels) < mean(&legacy) - 0.02,
        "median {median:.2}, lit {lit:.2}, mean {:.3}/{:.3}, settings {a:?}",
        mean(&pixels),
        mean(&legacy)
    );
    assert!(a.exposure > 1.5 && a.exposure < 3.2, "{a:?}");
}

#[test]
fn high_key_auto_preserves_bright_scene_intent_in_render() {
    let src = high_key_scene();
    let a = auto_tone(&src, &SourceInfo::default(), &DevelopSettings::default());
    let median = median_after(&src, a);
    let pixels = output_luma(&auto_render(&src));
    let legacy = output_luma(&render_with(&src, &legacy_settings(&src)));
    let bright = pixels.iter().filter(|v| **v > 0.25).count() as f32 / pixels.len() as f32;
    assert!(
        median > 0.5 && bright > 0.75 && mean(&pixels) > mean(&legacy) + 0.02,
        "median {median:.2}, bright {bright:.2}, mean {:.3}/{:.3}, settings {a:?}",
        mean(&pixels),
        mean(&legacy)
    );
    assert!(a.exposure > -2.0 && a.exposure < -0.7, "{a:?}");
}

#[test]
fn ordinary_underexposed_auto_reaches_middle_exposure_in_render() {
    let src = ordinary_underexposed_scene();
    let a = auto_tone(&src, &SourceInfo::default(), &DevelopSettings::default());
    let median = median_after(&src, a);
    let pixels = output_luma(&auto_render(&src));
    let legacy = output_luma(&render_with(&src, &legacy_settings(&src)));
    assert!(
        median > -0.25 && mean(&pixels) > mean(&legacy) + 0.02,
        "median {median:.2}, mean {:.3}/{:.3}, settings {a:?}",
        mean(&pixels),
        mean(&legacy)
    );
}

#[test]
fn low_key_auto_keeps_small_skin_highlight_distinct() {
    let src = dark_scene_with_skin_highlight();
    let a = auto_tone(&src, &SourceInfo::default(), &DevelopSettings::default());
    let pixels = output_luma(&auto_render(&src));
    assert!(a.exposure > 1.5 && a.exposure < 3.2, "{a:?}");
    assert!(pixels[4 * 64 + 4] > pixels[40 * 64 + 40] + 0.04, "skin/detail collapsed: {a:?}");
}

#[test]
fn backlit_auto_limits_foreground_brightening_and_recovers_tail() {
    let src = backlit_scene();
    let a = auto_tone(&src, &SourceInfo::default(), &DevelopSettings::default());
    let image = auto_render(&src);
    let pixels = output_luma(&image);
    let legacy = output_luma(&render_with(&src, &legacy_settings(&src)));
    let highlights = pixels.iter().filter(|v| **v > 0.98).count() as f32 / pixels.len() as f32;
    let legacy_highlights = legacy.iter().filter(|v| **v > 0.98).count() as f32 / legacy.len() as f32;
    assert!(a.exposure <= 2.0 && a.highlights < -50.0, "settings {a:?}");
    assert!(
        highlights <= legacy_highlights && mean(&pixels) < mean(&legacy) - 0.01,
        "highlight clip share {highlights:.2}/{legacy_highlights:.2}, mean {:.3}/{:.3}, settings {a:?}",
        mean(&pixels),
        mean(&legacy)
    );
}

#[test]
fn saturated_auto_reduces_rendered_colour_boost() {
    let a = auto_tone(&saturated_scene(), &SourceInfo::default(), &DevelopSettings::default());
    let image = auto_render(&saturated_scene());
    let chroma = output_chroma(&image);
    let legacy = output_chroma(&render_with(&saturated_scene(), &legacy_settings(&saturated_scene())));
    let mean_chroma = mean(&chroma);
    let legacy_mean = mean(&legacy);
    assert!(a.vibrance <= 4.0 && a.saturation <= 0.0, "{a:?}");
    assert!(mean_chroma < legacy_mean - 0.002, "rendered chroma {mean_chroma:.3}/{legacy_mean:.3}, settings {a:?}");
}

#[test]
fn neutral_auto_retains_gentle_rendered_colour_lift() {
    let a = auto_tone(&neutral_scene(), &SourceInfo::default(), &DevelopSettings::default());
    let image = auto_render(&neutral_scene());
    let chroma = output_chroma(&image);
    let max = chroma.into_iter().fold(0.0, f32::max);
    assert!(a.vibrance >= 12.0 && a.saturation >= 2.0, "{a:?}");
    assert!(max < 0.01, "neutral rendered chroma {max:.3}, settings {a:?}");
}

#[test]
fn invalid_pixels_never_make_auto_non_finite() {
    let mixed = Rgb32f::from_fn(32, 32, |x, y| match (x + y) % 7 {
        0 => [f32::NAN, 0.1, 0.1],
        1 => [f32::INFINITY, f32::NEG_INFINITY, 0.1],
        _ => [0.12, 0.1, 0.09],
    });
    let a = auto_tone(&mixed, &SourceInfo::default(), &DevelopSettings::default());
    assert!(finite_outputs(a), "{a:?}");

    let invalid = Rgb32f::filled(8, 8, [f32::NAN, f32::INFINITY, f32::NEG_INFINITY]);
    assert!(finite_outputs(auto_tone(&invalid, &SourceInfo::default(), &DevelopSettings::default())));
}

#[test]
fn empty_input_returns_neutral_finite_settings() {
    let a = auto_tone(&Rgb32f::new(0, 0), &SourceInfo::default(), &DevelopSettings::default());
    assert_eq!(a, AutoTone::default());
}
