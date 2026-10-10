# Installed Lightroom behavior study — 2026-10-10

## Result

Eight original numerical inputs were imported into an isolated Lightroom Classic catalog. Auto produced eight-slider edits for every input; repeating Auto preserved all 64 recorded tone/color values. Two inputs with exactly equal source RGB histograms produced materially different edits when spatial arrangement changed. Installed Classic also explicitly reports non-destructive Enhance edits without another DNG.

This is black-box behavior research, supported by public Adobe descriptions. No Adobe binary, model, preset, profile, camera matrix or asset was inspected, extracted or copied. Personal photographs were not imported or edited. Numerical edit receipts are reference measurements, not training labels or a claim of photographic quality.

## Experiment contract & evidence

- Installed Lightroom Classic **15.6**, build `202609251514-f839cde9`; Camera Raw **18.7**, build `2743`, native arm64; Metal Apple M6, GPU preference Auto. Version/hardware were read from System Info in the preceding inspection.
- Original flat RGB levels 32/128/224, ordered/shuffled grayscale ramps, color patches, plus clean/noisy synthetic RGGB DNGs. TIFF inputs are lossless 512×512; DNG inputs are 1024×1024, with original identity-XYZ synthetic sensor metadata. These charts are not photographs or a real camera calibration.
- [Generator](../tools/generate-adobe-probes.py), [input SHA-256 receipts](research/adobe-study-2026-10-10/inputs.json), [baseline edits](research/adobe-study-2026-10-10/baseline.json), [first Auto](research/adobe-study-2026-10-10/auto-first.json), [repeated Auto](research/adobe-study-2026-10-10/auto-repeat.json).
- Generator reproduced all eight study files byte-for-byte in a second external directory. Generated media/catalogs stay outside this repository. The generator requires an external output directory & rejects an existing inputs directory.
- Process Version **15.4**; initial eight tone/color sliders were all zero. Auto was invoked from Library with all eight probes selected, then invoked again. WB/profile defaults were not changed. No presets were imported.
- Receipts were read from the disposable catalog through SQLite `mode=ro`, selecting only input names, process version & whitelisted numeric edit values. Serialized catalog text was never executed; no catalog was edited with SQLite. This research capture does not add a SQLite dependency to Ember.
- Original startup catalog was reopened after experiments, without editing its contents. Lightroom was closed afterward. No Ember launch/install occurred.

## Auto observations

Slider order below: Exposure EV / Contrast / Highlights / Shadows / Whites / Blacks / Vibrance / Saturation.

| Input | First Auto values |
|---|---|
| Flat 32 | `1.25 / 7 / -100 / 88 / 50 / -60 / 12 / 2` |
| Flat 128 | `-1 / 7 / -99 / 77 / 50 / -60 / 20 / 4` |
| Flat 224 | `0.9 / 7 / -98 / 66 / 46 / -27 / 20 / -3` |
| Spatial grayscale ramp | `0.38 / 6 / -76 / 53 / 5 / -12 / 20 / 3` |
| Shuffled grayscale ramp | `0.4 / 7 / -56 / 3 / 50 / -20 / 20 / 4` |
| Color patches | `-0.56 / 5 / -13 / 19 / -10 / -1 / 8 / 2` |
| Synthetic clean Bayer | `0.45 / 4 / -92 / 16 / -9 / -37 / 20 / 2` |
| Synthetic noisy Bayer | `0.21 / 7 / -46 / 57 / -8 / -22 / 20 / 1` |

1. **Repeated command stability:** all 64 tone/color values were unchanged on the second Auto command. This observes command idempotence for these inputs, not cross-device/release determinism or proof a model ran twice.
2. **Equal full-resolution histograms do not imply equal Auto:** both ramps contain each grayscale RGB triplet 0–255 exactly 1,024 times. Only arrangement differs. Shadows changes **53→3**, Whites **5→50**, Highlights **−76→−56**. Source histogram equality was verified independently.
3. **Interpretation:** spatial structure and/or spatial preprocessing matters somewhere in Adobe's path. A thumbnail resize can itself alter the histogram of a shuffled image. This experiment does not isolate CNN semantic recognition, feature extraction, receptive field or model topology.
4. **Degenerate inputs are not an exposure-quality benchmark:** flat charts produced non-monotonic Exposure values & extreme tone values. Do not fit a desired exposure curve or photographic quality claim to them.
5. **Noise changes Auto in this setup:** matched synthetic clean/noisy raw scenes produced different edits. This establishes sensitivity under these defaults; it does not prove denoise should always precede Auto on real shoots.

The public [Auto API guide](https://developer.adobe.com/firefly-services/docs/lightroom/guides/auto-tone/) describes a learned eight-slider predictor; its cloud endpoint does not establish where installed Classic runs. [Auto research](adobe-auto-research.md) records Adobe facts, Ember's statistical baseline & further probes.

## Denoise observations & research

Both original DNGs imported & showed image previews in Import; the noisy DNG also displayed in Develop. These previews were observed firsthand, with no exported render receipt. Selecting the noisy DNG, resetting its edits, opening Detail & trying its Denoise control did **not** produce a verified activation. Coordinate control of this custom panel was inconclusive; this is not a finding that synthetic DNGs are unsupported. There is no Denoise output, noise/detail score, runtime or Amount curve from this experiment.

`Photo > Enhance` opened an installed-product notice stating, in paraphrase, that Denoise, Raw Details & Super Resolution have moved into Detail & apply non-destructively without creating another DNG. This directly observed 15.6 behavior supersedes older new-DNG assumptions. No Adobe screenshot was saved or committed.

Adobe's [Denoise Demystified](https://blog.adobe.com/en/publish/2023/04/18/denoise-demystified) discloses a convolutional model trained on noisy raw/clean RGB patches with simulated & measured noise, jointly denoising & demosaicing. [Current Classic help](https://helpx.adobe.com/lightroom-classic/desktop/process-and-develop-photos/enhance-details.html) documents GPU processing & broader raw support. [Denoise research](adobe-denoise-research.md) separates these disclosures from undisclosed weights/operators & historical workflow details.

No disconnected-network execution test was performed. No network/security settings were changed. Local-processing documentation is recorded in [the preceding inspection](lightroom-local-processing.md); a live-network input/output run alone cannot prove absence of a server dependency.

## Independent Ember direction

1. **Keep measurable local Auto baseline.** Current `auto.rs` reduces a 512px proxy into order-independent luminance/chroma distributions. For these same-size histogram-equivalent TIFFs, static source analysis predicts equal statistics; no Ember runtime comparison was run. Add a candidate with multi-scale spatial/patch features or a small permissively licensed image encoder, then compare against this baseline on consented held-out shoots. Candidate output must remain bounded & reviewable.
2. **Use Ember edits as ground truth.** Lightroom receipts above are behavior references; imported Lightroom adjustments remain weak relative-style labels because renderer semantics differ. Learn Personal Auto from validated Ember edits & require a held-out preference improvement over deterministic Auto.
3. **Preserve command stability.** Cache keys must bind input pixels, crop, WB/profile, feature/model version & edit semantics. Repeated accepted Auto should not accumulate adjustments. Crop/profile/WB sensitivity still needs its own controlled experiment.
4. **Qualify denoise independently.** First measure a compact post-demosaic RGB model against existing guided-filter NR, with noise reduction, detail/blur preservation & CPU/Metal memory/latency receipts. Label it accurately. Joint raw→RGB denoise requires a separate mosaic/normalization/color/tile contract & independently licensed weights; Adobe's weights are unnecessary.
5. **Respect non-destructive workflow.** Keep originals authoritative, expose Amount & reversible enable/disable, cache derived pixels separately, & rerun image-dependent masks/healing when denoise changes their input. Do not require a new user-visible DNG solely to mirror an obsolete Adobe workflow.

Next execution deliverable: spatial Auto candidate contract & independent denoise qualification, with measured quality on original/consented images before shipping.
