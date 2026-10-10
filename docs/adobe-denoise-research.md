# Adobe Denoise research

**Scope/date:** clean-room study of public Adobe documentation, 2026-10-10. No Adobe bundle, model, photo or asset was inspected or downloaded. Statements below separate Adobe disclosures from inference.

## Public Adobe disclosures

Primary source: [Denoise demystified](https://blog.adobe.com/en/publish/2023/04/18/denoise-demystified) (Eric Chan, 2023-04-18).

- Denoise jointly demosaics sensor mosaics & removes noise, trained from noisy Bayer raw toward clean RGB rather than a documented post-demosaic denoiser.
- Adobe names model class only: **deep convolutional neural network**; topology, tensors, tiles, precision, operators, format & weights remain unknown.
- Training uses millions of paired high/low-noise patches, noise simulation/data augmentation & large dark-frame data for shadow pattern noise.
- The 2023 blog says Denoise also applies Raw Details, carries edits into a new DNG & zeros Manual Luminance/Color Noise Reduction; treat this as historical, not current contract.
- Adobe recommends Denoise before healing, Remove & masks; review masks/spots & tone after shadow cleanup. It calls Denoise GPU intensive & cites Tensor Cores/Apple silicon; these are not Ember measurements.

Related primary source: [Enhance Details](https://business.adobe.com/blog/the-latest/enhance-details) (Adobe, 2019-02-12) describes an earlier CNN demosaic feature, two separate models (Bayer & X-Trans), over one billion training examples, Core ML/Windows ML execution & up to 30% Siemens-Star chart resolution improvement. Those details concern Enhance Details, not Denoise metrics or Denoise architecture; do not transfer them to Denoise.

## Current product contract

[Current Lightroom Classic Enhance Details help](https://helpx.adobe.com/lightroom-classic/desktop/process-and-develop-photos/enhance-details.html) (page updated 2026-05-01) currently says:

- Enhance runs in background & is GPU intensive. It says Apple Neural Engine does **not** support AI Denoise on macOS.
- Denoise accepts Bayer/X-Trans mosaic raw plus several additional raw/DNG families: linear DNGs made inside Lightroom/ACR (including HDR/panorama DNG), DNG proxies, Leica Monochrom, Canon sRAW/mRAW, Nikon small raw, Fujifilm non-X-Trans raw, Sony ARQ, Pentax Pixel Shift, Foveon, Apple ProRaw DNG & Samsung Expert Raw. This expanded product list does not reveal whether one model, adapters or separate models serve each format.
- Denoise remains a Detail-panel intensity control & Adobe recommends it before other tools, including AI masks & Remove. Enhance output cannot be enhanced again.
- Current Help does **not** say that Denoise creates a new DNG. It explicitly reserves “select Enhance to create an enhanced DNG” for Raw Details. Therefore current Lightroom Classic’s apparent no-extra-file, non-destructive Denoise workflow must be treated as current product behavior, while 2023’s new-DNG statement stays historical.

Installed behavioral observation (Lightroom Classic **15.6**, 2026-10-10): opening **Photo > Enhance** showed: “The Enhance features have moved to the Detail panel. Denoise, Raw Details, and Super Resolution can now be applied non-destructively without creating a new DNG file.” This current UI statement resolves current no-DNG behavior for all three Enhance features, despite Help still containing Raw Details’ older “create an enhanced DNG” wording. Denoise activation was not completed; this observation establishes workflow/file behavior only, with no quality, latency or memory claim. No screenshot or Adobe asset was retained.

Help documents supported formats & workflow, but disclose no weights, model topology, raw normalization, crop/tile size, output tensor contract, loss, dataset license, model license, reproducible code, hardware kernel or numeric quality metric. Adobe's blog examples are qualitative; they do not establish Ember parity.

## Ember mapping

Ember currently has no AI Denoise implementation; [ROADMAP.md](../ROADMAP.md) keeps AI Denoise under M7 as open & records model strategy as a blocker. Existing [pipeline local processing](../crates/pipeline/src/local.rs) is classical RGB noise reduction: edge-aware guided filtering of log luminance plus chromaticity blur. [Render ordering](../crates/pipeline/src/lib.rs) performs white balance, defringe, spots & redeye, then this noise reduction. That is useful baseline behavior, but it is not Adobe's jointly trained raw-mosaic-to-RGB operation.

Implications:

1. A first Ember experiment should be explicitly **post-demosaic RGB denoise** so it fits current render stages with bounded change. It may improve noise/detail, but cannot be described as Adobe-like raw joint demosaic/denoise.
2. A raw-domain experiment needs a new clean contract before model selection: supported mosaic patterns, black/white levels, clipping, gain/WB position, output color space, tile overlap & DNG/cache persistence. Current public Adobe docs do not supply these values.
3. Preserve Ember's non-destructive source: write denoised intermediates to disposable cache/catalog state, while raw source & edit history remain authoritative. This fits current edit-panel/non-destructive intent without introducing a DNG; Adobe's 2023 new-DNG behavior is historical & not an implementation requirement.
4. Apply any future AI denoise before image-dependent healing/masking stages, then re-evaluate existing masks/spots & shadow/tone controls. Keep current Manual NR controls independent so users can compare or combine them intentionally.
5. Do not infer Metal acceleration from Adobe's Apple silicon guidance. Ember must measure its own wgpu Metal path, Windows CPU fallback & memory behavior; current Ember has no Denoise latency, memory or held-out-quality result.

## Independent pure-Rust experiments

Use only original procedural/generated samples or user-consented disposable catalog material. Keep model code, weights & training data under separate license receipts; require permissive terms and SHA-256 pinning before any artifact enters Ember.

1. **Baseline first:** compare current guided luminance/chroma NR against an identity path on generated Bayer-like scenes with shot noise, read noise, row/pattern noise & shadow lifts. Include defocus and motion-blurred edges, fine texture, repeated patterns & small color patches. Report edge/detail retention, chroma error, shadow color stability & blur preservation; do not turn a smooth output into a quality win.
2. **Small post-demosaic residual CNN:** train outside Ember on independently generated noisy/clean RGB pairs, export a small convolution-only model in a permissive, immutable format, then port only operators already available in Ember's pure-Rust stack. Keep residual strength/Amount explicit, clamp non-finite output, and compare against current NR. This tests product value without claiming raw joint demosaic.
3. **Raw-domain feasibility spike:** only after contract approval, train or obtain an independently licensed compact model for normalized mosaic input → linear RGB. Start with one Bayer layout & fixed tile/overlap; add X-Trans or other formats only after separate data/quality gates. A model may be rejected when it hallucinates texture or sharpens intentional blur even if pixel metrics improve.
4. **Qualification gate:** record model-code, weight & training-data licenses separately; exact source revision; artifact size/SHA-256; operator inventory; input/output shapes, range & normalization; non-finite/error behavior; CPU and Metal cold/warm p50/p95 latency; peak memory; face/culling-adjacent false-detail review; held-out quality on generated & consented samples. Adobe's qualitative examples and ISO rule of thumb stay context only.
5. **Review workflow:** use disposable catalog renders & original generated samples for paired before/after review. Keep a manual accept/reject path and preserve intentional blur, grain & texture. Ship no accuracy claim until Ember measurements exist.

### Minimal next action

Create one qualification task for a small, permissively licensed **post-demosaic** CNN candidate (or an original model) using generated noisy/clean RGB pairs. Freeze input range, output color space, Amount blend, blur-preservation cases, license/SHA receipt & CPU/Metal measurement plan. If that gate passes, design raw-domain support separately; no public Adobe architecture or weight is sufficiently specified for direct reproduction.
