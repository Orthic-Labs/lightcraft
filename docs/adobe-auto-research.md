# Adobe Auto research

This note uses public Adobe descriptions & static review of current Ember source. It does not inspect Adobe bundles, decompile models, copy Adobe assets or claim Adobe parity.

## Facts from Adobe

- Adobe's [Auto Tone API guide](https://developer.adobe.com/firefly-services/docs/lightroom/guides/auto-tone/) says Auto Tone uses an AI/ML model trained to adjust Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Saturation & Vibrance from image content. The guide gives an asynchronous service request/response, but no feature schema, model version, preprocessing, target distribution or exact slider mapping.
- Adobe's [December 2017 announcement](https://blog.adobe.com/en/publish/2017/12/12/announcing-december-update-lightroom) says Auto was reworked around an Adobe Sensei neural network, compared photos with tens of thousands of professionally edited photos & shipped across Lightroom products. This is historical product description, not a current model specification.
- Adobe's public Auto material does not establish where inference runs, whether metadata participates, how clipping/high-key intent is treated, or whether repeated calls are numerically deterministic.

## Current Ember behavior

`crates/pipeline/src/auto.rs` is a deterministic statistics baseline:

- `auto_tone` fits a Box proxy to 512×512, applies scene-linear preprocessing with only current WB, collects finite luminance EV values (`log2(Y / 0.18)`) & Oklab chroma, then uses 1%, 5%, 50%, 95% & 99.5% percentiles.
- Low-key, high-key & backlit branches select fixed targets/damping; Exposure, tone sliders, Vibrance & Saturation are hand-written bounded formulas. Existing tone sliders are intentionally ignored. This is useful independent Rust behavior, but it is not evidence of Adobe's neural network or its professional-edit target.
- `auto_wb` is a separate grey-world estimator. It is not called by `auto_tone`.
- Existing regression fixtures cover key scenes, baseline exposure, clipping/backlight, saturation, neutral colour & non-finite pixels. They validate Ember invariants, not Adobe parity.

## Hypotheses to test

These are testable hypotheses, not Adobe facts:

1. Adobe Auto uses semantic/compositional features in addition to global tone statistics; two images with matched histograms but different spatial arrangements or content classes may receive different values.
2. Auto may be approximately stable under uniform scene exposure changes after slider compensation, but clipping, high-key intent, faces/skin, sky and backlight may break simple histogram invariance.
3. Auto may use camera/profile metadata as a prior, or may depend only on decoded pixels. Same pixels with changed metadata can distinguish these paths.
4. Output values may be quantized or model-version dependent even when source bytes are unchanged.

## Disposable Auto probe matrix

Use synthetic inputs & a disposable catalog only. Reset edits, profile & WB unless probe varies them. Record source bytes/hash, metadata, app/model versions, command, output settings, output bytes/hash & every slider value.

| Probe | Input pair or sweep | Record | Distinguishes |
| --- | --- | --- | --- |
| Histogram permutation | Same pixels and histogram, spatially permuted; include sky/skin-like, edge-rich & flat regions | Eight Auto values plus rendered proxy | Global statistics versus spatial/content features |
| Key ladder | Neutral synthetic scene scaled by −4…+4 EV, with unclipped and clipped variants | Slider vector, effective median, percentile/tone summaries | Exposure compensation, clipping thresholds & branch transitions |
| Scene intent | Low-key, high-key, backlit, grey card, saturated patches, skin-like patch, specular tail | Slider vector & clipped-pixel share | Semantic/key protection versus percentile rules |
| Framing | Same scene cropped, padded, rotated & mirrored | Slider vector, crop metadata | Frame/composition dependence |
| Noise/metadata | Same pixels with controlled Poisson/read noise, ISO/exposure metadata changes, camera-tag changes | Slider vector & rendered output | Pixel-driven versus metadata-driven behavior |
| Repeat/version | Same input repeated, then same input across Adobe versions/GPU paths | Exact slider values, output hash, version/GPU | Determinism & model drift |

For each run compare Adobe values with current `auto_tone` values as descriptive baselines only. Do not tune Ember formulas toward one scene without a held-out matrix.

## Implications for independent Rust Auto

- Keep current `auto_tone` as documented deterministic baseline with finite/bounded outputs & regression fixtures. Add a versioned model interface only when model, feature contract & permissive weights are available; do not label percentile behavior Adobe Auto.
- Keep Auto input preparation explicit: proxy size/filter, WB/profile state, metadata policy, invalid-pixel handling & output bounds belong in a versioned contract.
- Make Auto outputs inspectable as one vector with source/model/version binding. Preserve current edits until explicit apply, and make any future learned model replaceable without changing catalog semantics.
- Require same-input reference outputs plus held-out measurements before any parity statement. Adobe marketing descriptions alone cannot support quality claims.

## Independent spatial candidate source

Offline Personal Auto feature-v2 now appends 12 original photometric statistics to existing 16 scalar fields: EV mean, EV population standard deviation & mean Oklab chroma in each 2×2 cell of upright 512px WB-only proxy. Same-histogram layouts can supply different spatial vectors. This is an independent hypothesis for residual style evaluation, not a reproduction of Adobe's predictor, a semantic classifier or evidence of improved Auto. Production `auto_tone` & its scene-control formulas stay unchanged; [Personal Auto protocol](personal-auto-evaluation.md) requires shoot-disjoint rendered/preference comparisons before promotion.

## Denoise cross-reference

Denoise facts, including Adobe's disclosed joint demosaic/denoise CNN, current Lightroom Classic format/workflow contract & Ember implications, live in [Adobe Denoise research](adobe-denoise-research.md). This note keeps Denoise probes out of Auto research to avoid duplicating or contradicting that contract.

## Auto sources

- [Adobe: Use Lightroom API Auto Tone](https://developer.adobe.com/firefly-services/docs/lightroom/guides/auto-tone/)
- [Adobe: Announcing the December Update to Lightroom](https://blog.adobe.com/en/publish/2017/12/12/announcing-december-update-lightroom)
