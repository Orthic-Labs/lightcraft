# Offline denoise qualification

Status, 2026-10-10: source-only diagnostic protocol. Production Ember has classical RGB luminance/chroma noise reduction; no learned denoiser is qualified. Adobe's disclosed joint raw demosaic/denoise & current non-destructive workflow are studied separately in [Adobe Denoise research](adobe-denoise-research.md).

## Procedural baseline

```text
lightcraft-cli ai denoise baseline --hardware M6_CPU --source-revision REVISION --out NEW_REPORT.json --size 128 --repeats 3
```

Run through generated RightKit native Actions. `--size` accepts 64..512 & `--repeats` accepts 2..10 warm calls after one first call; defaults are 128 & 3. Hardware/revision labels are supplied metadata, not independently verified machine/source identities. Output is create-new, capped numeric JSON. No input files, personal photos, catalog, model, provider or Adobe assets are accessed.

Original procedural linear Rec.2020 patterns cover flat shadows, color patches/gradient, hard step edges, fine texture & intentionally soft edges. Fixed-seed synthetic signal-dependent/read noise supplies repeatable noisy inputs; its constants do not represent a calibrated camera. Identity output & production `pipeline::local::denoise` at amounts 0, 25, 50, 75 & 100 are compared against each clean pattern. Both luminance & color NR receive the amount; remaining detail controls keep defaults. Source/output longest sides match, so preview scaling is not part of this condition.

Metrics include RGB MSE/RMSE, peak-1 PSNR, signed channel bias & gradient RMSE against clean input. Zero MSE has no finite PSNR & is represented explicitly. Finite production output is measured without extra clipping; out-of-range samples are counted & nonfinite output fails. Identity establishes injected error; clean edges, texture & soft edges expose smoothing or invented-detail distortion. Pixel metrics alone cannot determine photographic preference.

Receipts bind dimensions plus little-endian F32 clean/noisy/output pixels by SHA-256 without storing images. First-call & repeated p50/p95 timing covers NR only; fresh image clones, procedural generation, metrics & hashing are outside that scope. First call is not cold-process/cache timing. Repeated-output maximum difference & tolerance expose numerical drift. All reports remain `UNQUALIFIED`, including numerically perfect or deterministic results.

## Learned RGB candidate gate

[Candidate record](models/denoise-candidates.md) selects full NAFNet-SIDD-width32 as first architecture hypothesis, with pinned source/config & separately unresolved first-party checkpoint permission.

1. Pin first-party architecture/config/checkpoint identity, code licence, checkpoint permission & training-data scope separately. A converted-model card's licence label cannot alone prove first-party checkpoint rights. Record original vs converted precision/operator differences.
2. Establish exact color/transfer, channel order, input range, normalization, pad/crop, tile overlap, global-statistic & output-clamp contracts. Display RGB models cannot silently receive Ember's scene-linear Rec.2020 buffer. Define any color conversion & compare against an independent reference.
3. Compare whole-image vs tiled output, boundaries, bright/shadow color, skin, texture, soft focus & hallucinated structure. Record amount blending separately; intensity is an Ember contract unless model supports it directly.
4. Measure first/warm preview & full-resolution latency, peak memory, cancellation, cache identity & source preservation on named Mac/Windows targets. Operator availability or advertised mobile latency does not qualify Ember performance.
5. Evaluate consented shoot-disjoint camera/ISO strata, including intentional blur, alongside classical NR. Freeze settings on training/validation shoots; use held-out rendered comparison & human preference before any promotion.

An initial RGB candidate is post-demosaic. It cannot establish raw-mosaic reconstruction parity. Future joint raw denoise requires separate sensor/noise/calibration data & camera coverage. Shipping also requires generated CI, exact-artifact hidden native QA & explicit review/acceptance behavior.

## Next evidence

Current harness & regression tests have not executed on this revision. Restore fork Actions, produce baseline receipts, then compare a rights-cleared RGB candidate against identical procedural & consented photographic conditions. Keep source files & catalog authoritative; experimental intermediates belong in disposable cache state.
