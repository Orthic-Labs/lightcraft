# Auto tone

`develop.auto` (`crates/pipeline/src/auto.rs`) is deterministic & closed-loop. Same pixels, same source kind, same white balance → same eight values, byte for byte, on every platform (f32 statistics, fixed sample stride, fixed iteration order & count, no clock, no randomness).

## Stages

1. **Proxy.** Box-fit to 512 px, white balance & scene-linear preparation as the pipeline does it, then every finite pixel's scene EV (`log2(Y / 0.18)`) & Oklab chroma.
2. **Scene key** (`SceneKey`). Low-key, high-key & backlit are continuous weights in 0..=1 from smooth steps on the EV percentiles, never branches. Two frames of one burst that straddle a threshold get nearly the same values (regression `burst_frames_across_a_key_threshold_get_nearly_the_same_auto`: the old branch moved exposure by 0.9 EV between frames 6 % apart).
3. **Exposure.** An exact rule: exposure is a gain before the tone curve, so `target_key − median` lands the median where the key wants it (0 EV ordinary, −0.9 low-key, +0.65 high-key, blended). Backlit scenes cap exposure near +2 EV so recovery handles the tail instead of lifting a dark foreground several stops.
4. **Shaping fit** (`auto::fit`). Contrast, highlights, shadows, whites & blacks are fitted against a per-pixel model of the finish stage: finish's own highlight/shadow log-luminance offsets (the pixel's luminance stands in for the edge-aware base plane) followed by the source's real tone map (`ToneMap::camera` / `new` / `display`, chosen as `finish` chooses). Targets on the rendered histogram: bright tail reaches white without clipping (also measured in EV against the tone map's white point, which stays informative where the output saturates), blacks reach near black, ordinary mid-tone spread, median kept where exposure put it. Coordinate descent in fixed slider order with a fixed step schedule (16, 8, 4, 2) over four passes, bounded to Auto's ranges (highlights only recover, shadows only lift), with regularisation that keeps sliders modest where they buy nothing. Cost per evaluation: ≤ 4096 samples × two LUT lookups.
5. **Colour.** Vibrance & saturation from the 90th-percentile chroma headroom (unchanged rules).

Why closed-loop: the previous rules mapped percentiles straight to slider values & assumed each slider's effect was linear & independent. Tone-curve tuning silently detuned Auto, & a display-referred JPEG (clips at 1.0) got the same recovery as a raw with a Reinhard shoulder. The fit reads the tone map, so Auto follows the pipeline (regression `fitted_sliders_track_the_tone_map_of_the_source_kind`).

## Pieces other features reuse

`auto::scene_population` (the proxy's sorted scene EVs, percentiles & chroma), `SceneKey`, `auto::fit::Targets` / `auto_targets` / `fit_with`, `encode` & `tone_map` are public: look targets ([`look-targets.md`](look-targets.md)) feed measured statistics into the same fit; Personal Auto adds a learned residual on top of the result.

## Contract for offline work

`auto::REVISION` names the rules; Personal Auto receipts bind to it & reject stale baselines. Changing any constant above is a new revision.

## Next

Face / centre weighting of the exposure target (YuNet is in `crates/segment`), raw clipped-channel share capping recovery, shades-of-grey Auto WB, scene priors from DINOv2 embeddings blended by probability, & spatial / face-weighted look targets. Evaluation: `docs/personal-auto-evaluation.md` (shoot-disjoint preference against this baseline).
