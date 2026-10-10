# Offline denoise qualification

Status, 2026-10-10: procedural CPU baseline measured through generated CI. Production Ember has classical RGB luminance/chroma noise reduction; no learned denoiser is qualified.

## Procedural baseline

```text
lightcraft-cli ai denoise baseline --hardware M6_CPU --source-revision REVISION --out NEW_REPORT.json --size 128 --repeats 3
```

Run through generated RightKit native Actions. `--size` accepts 64..512 & `--repeats` accepts 2..10 warm calls after one first call; defaults are 128 & 3. Hardware/revision labels are supplied metadata, not independently verified machine/source identities. Output is create-new, capped numeric JSON. No input files, personal photos, catalog, model, provider or reference-product assets are accessed.

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

Fork Actions are restored & now produce baseline receipts on candidate builds. Keep source files & catalog authoritative; experimental intermediates belong in disposable cache state.

### CI evidence, run 38055623316

[Mac & Windows candidate receipts](https://github.com/Orthic-Labs/lightcraft/actions/runs/38055623316) completed from source revision `93678dd1b9977d6ee509679be164fe2ec3b44113`. Both used 128×128 procedural input, one first call plus three warm calls, CPU-only processing, zero repeatability errors & zero out-of-range channels. Receipts remain alwaysUNQUALIFIED.

| Case | Identity MSE / gradient / PSNR | Amount 100 MSE / gradient / PSNR | Best sampled amount by MSE |
| --- | --- | --- | --- |
| Flat shadows | 1.734e-5 / 5.717e-3 / 47.61 dB | 1.718e-6 / 1.558e-3 / 57.65 dB | 100 |
| Color patches/gradient | 1.225e-4 / 1.564e-2 / 39.12 dB | 6.916e-3 / 3.154e-2 / 21.60 dB | 0 |
| Hard step edge | 1.618e-4 / 1.796e-2 / 37.91 dB | 2.615e-4 / 8.072e-3 / 35.83 dB | 50 |
| Fine texture | 1.530e-4 / 1.744e-2 / 38.15 dB | 1.108e-3 / 4.708e-2 / 29.56 dB | 25 |
| Intentional soft edge | 1.492e-4 / 1.723e-2 / 38.26 dB | 7.502e-5 / 4.395e-3 / 41.25 dB | 75 |

Mac host metadata was Apple M1 (Virtual), 3 CPUs & 7,516,196,864 bytes memory; nonzero-NR warm p50 ranged 0.391–0.482 ms & p95 0.392–0.784 ms. Windows metadata was AMD EPYC 7763 64-Core Processor, 4 CPUs & 17,174,360,064 bytes; warm p50 ranged 1.842–2.319 ms & p95 1.955–2.436 ms. Mac & Windows CLI hashes were fbce434888b0892eb8c8cbb5b90a0625d5246d350bf9dc8c404fcaeb7170e2c3 & 30b3e43909d95f0320c92f671e23da40bc114e3180ec05f841c3c2533ea54915.

Cross-platform pixel SHA-256 values are not bit-identical, including noisy/output buffers & one soft-edge clean buffer, so receipts bind each platform independently. Results describe synthetic RGB guided-NR behavior only; they provide no photographic, learned-model or Metal evidence.

Next compare a rights-cleared RGB candidate against identical procedural & consented photographic conditions.
