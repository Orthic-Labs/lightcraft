# Lightroom Auto & Denoise — local processing reference

Read-only check, 10 October 2026: installed Lightroom Classic reports **15.6**, native arm64,
Camera Raw **18.7**, Metal GPU acceleration & automatic GPU preference. System Info was inspected;
no photo was imported, edited, rated or deleted. Lightroom was quit after inspection.

## Processing evidence

- **Auto Tone:** Adobe describes neural-network analysis trained on professionally edited photos.
  Training location does not establish where per-photo inference runs. Martin Evening's original
  Adobe Press update guide explicitly describes Auto calculations as offline, without a live
  internet connection. This guide documents December 2017 behavior; it is not an offline execution
  test of installed Classic 15.6.
- **AI Denoise:** Adobe's current troubleshooting guide identifies Denoise models included with
  Lightroom Classic & explains version conflicts when older Classic loads Camera Raw's installed
  models. Current Enhance documentation describes GPU-intensive processing, available GPU/eGPU use
  & no Apple Neural Engine support for AI Denoise on macOS. This supports local model inference,
  rather than mandatory server processing.

Installed-version observation plus documentation establishes a local-processing reference. It does
not measure latency, memory, quality, network traffic or Ember parity. Future runtime comparison
requires isolated connectivity & explicitly supplied, consented photos.

## Primary sources

- [Adobe's Auto announcement](https://blog.adobe.com/en/publish/2017/12/12/announcing-december-update-lightroom).
- [Martin Evening, Adobe Press update guide](https://ptgmedia.pearsoncmg.com/imprint_downloads/peachpit/peachpit/promo/lrclassic/lrcc-12-2017.pdf), Auto Settings discussion on PDF page 3.
- [Adobe: troubleshoot Denoise issues](https://helpx.adobe.com/lightroom-classic/desktop/technical-support/workflow-issues/denoise-issues/troubleshoot-denoise-issues.html).
- [Adobe: Enhance details](https://helpx.adobe.com/lightroom-classic/desktop/process-and-develop-photos/enhance-details.html).

Ember can pursue both Auto & denoise locally. Select permissive model weights, then qualify actual
quality & target-platform latency; Adobe's implementation proves no parity claim for Ember.
