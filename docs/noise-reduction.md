# Noise reduction

Status, 2026-10-10: classical, deterministic, CPU & GPU equivalent. Auto Noise Reduction measures each photo's noise & sets the sliders; the filters themselves are unchanged (see *Next*).

## Render path (`pipeline::local::denoise`, GPU `render::denoise`)

Luminance: self-guided filter of log luminance (radius & edge threshold from *Noise Reduction* & *Detail*), blended by `k = √amount`. Colour: chromaticity (`rgb / Y`) Gaussian-blurred by *Color Noise Reduction* & *Smoothness*, mixed back by amount × (1 − *Detail* / 2), luminance re-applied. Radii scale with the preview's downsampling so previews & exports match. `docs/denoise-qualification.md` holds the procedural benchmark (`lightcraft-cli ai denoise baseline`).

## Noise estimate (`pipeline::noise::estimate`)

Fits `var(Y) = a·Y + b` (shot + read noise, scene-linear luminance, middle grey = 0.18) on the centre 1536 px window of the native-resolution image:

1. 8 × 8 tiles; per tile the mean & Immerkær's residual variance (3 × 3 Laplacian `[1 −2 1; −2 4 −2; 1 −2 1]`, `σ² = Σr² / 36n`), which is unbiased for white noise & only *grows* with texture or edges.
2. Tiles fall into 1 EV intensity bands (−7 … +4 EV around grey). In each band with ≥ 6 tiles the quietest 20 % define the noise floor: texture & edges drop out because they can only raise a tile above it.
3. Weighted least squares over band floors, `a, b ≥ 0`. `luminance_mid = σ(0.18) / 0.18`, `luminance_shadow = σ(0.0225) / 0.0225` (three stops down), `chroma_mid` = the same floor on `R/Y` & `B/Y` in mid-tone tiles.
4. A proxy that averaged `scale²` native pixels per pixel has `scale²` less variance; `estimate(src, scale)` undoes it. `develop.autoNoise` reads the original (`SourceLevel::Full`) so `scale = 1`.

Refuses (returns `None`, never panics) images under 16 px, all-black or non-finite frames, & frames without a band of flat tiles. Repeat-stable: fixed tiling, bands, iteration order & f64 sums.

## Auto Noise Reduction (`develop.autoNoise`, Photo ▸ Auto Noise Reduction)

`noise::auto_noise_reduction(estimate, iso, raw)`:

| Input | Luminance NR | Colour NR |
| --- | --- | --- |
| Measured | `24 × max(0, 0.67·log2(mid / 0.010) + 0.33·log2(shadow / 0.020))` | raw floor 25 (rendered 0) + `20 × max(0, log2(chroma / 0.010))` |
| ISO only (no flat patches) | `16 × max(0, log2(ISO / 400))` | floor + `12 × max(0, log2(ISO / 400))` |
| Neither | 0 | floor |

Clamped to 0..=80 & 0..=100, rounded. Clean constants (`CLEAN_LUM`, `CLEAN_CHROMA`) are a base-ISO raw's relative noise; calibration below. *Detail* / *Contrast* / colour *Detail* / *Smoothness* keep their values. `dryRun: true` returns the estimate without editing. One undo step. Runs separately from `develop.auto`.

Calibration on the CC0 corpus (`cargo xtask corpus --download`, 56 raws, native-resolution estimate, 2026-10-10): base-ISO raws (Canon 5D3/5DS R/80D/R8, Nikon D7500/D7000, Pentax K-3/K-5 IIs/K10D, Fuji GFX, Sony A7R II / A7 IV) measure `luminance_mid` 0.004–0.007 & `chroma_mid` 0.002–0.009 → luminance 0, colour 25; ISO 640–1250 (Canon R100 / M50 / 7D, Sony A7 IV) measure 0.015–0.018 & 0.019–0.025 → luminance 20–25, colour 43–52; a Pixel 2 XL DNG at ISO 51 (small photosites) measures 0.017 → 11. Extrapolated by the fitted model, ISO 6400 lands near 50–60. The corpus has no raw above ISO 1250: that range is a model extrapolation until measured. Two outliers (`cr2-canon-6d` at ISO 100 measuring 0.036, `arw-sony-a7m3-uncompressed` five times its compressed sibling) need a look at the scene & decoder before they count as calibration points; `nrw-nikon-b700` decodes wrongly (purple) & is excluded.

## Next (ordered)

1. **Variance-stabilising transform.** Noise is signal-dependent; the luminance filter works in log luminance, which over-weights shadow noise & under-weights highlight noise. With `a, b` known, the generalised Anscombe transform `2·√(a·Y + b + 3a²/8) / a` makes it uniform; filter there, invert. Needs the same change in `finish.wgsl` / `render::denoise` for CPU–GPU equivalence (`tests_window::noise_reduction_does_not_depend_on_the_window` & the GPU equivalence tests guard it).
2. **Luminance-guided chroma.** Gaussian chromaticity blur bleeds colour across edges. A guided filter of `R/Y`, `B/Y` with log luminance as guide keeps edges; the GPU already has `guided_pre` / `guided_ab` / `guided_apply` for the self-guided case, a cross-guided variant adds one kernel.
3. **Non-local means at high ISO.** Above ~40 luminance NR, patch matching (7 × 7 patches, 21 × 21 search, VST domain) beats a guided filter by a wide margin on fine texture. Pure Rust, deterministic; GPU kernel later.
4. **Raw-domain denoise.** Before demosaic (`RawImage::develop`), per CFA plane in the VST domain: noise is simplest there (uncorrelated per photosite). Largest quality gain for raw; separate raw-cache key & qualification (`docs/denoise-qualification.md` → joint raw).
5. **Learned denoiser.** Train a compact NAFNet-width16-class network on *synthetic* Poisson–Gaussian noise added to clean CC0 raws using the `a, b` model above (no paired dataset, no third-party checkpoint rights); inference on Candle CPU for bit-stable receipts, Metal within tolerance. Gate per `docs/denoise-qualification.md`.
