# Photo assessment experiment

Read-only, provider-neutral contract & native OpenRouter transport. No catalog/edit/flag API is available to this crate. All hosts expose `ai.models`, `ai.schema` & `ai.validate`; native CLI adds `ai compare` with rendered references & receipts. Current `develop.auto` stays deterministic.

## Comparison

Use a CI-qualified `lightcraft-cli` binary. Configure `OPENROUTER_API_KEY` in its process environment securely (never in committed files or shell history), then:

```text
lightcraft-cli ai compare --demo --out /tmp/photo-ai-run --repeat 2 --budget-usd 0.50
lightcraft-cli ai compare /path/to/photo.jpg --out /tmp/photo-ai-one --model deepseek/deepseek-v4.1-flash --repeat 3
lightcraft-cli ai compare --demo --out /tmp/photo-ai-reference --prepare-only
```

Explicit files are opened in a fresh ephemeral session; saved libraries/folders are unsupported. Only metadata-free 512px PNG proxies leave process. Tone controls are reset to zero for input; WB/profile & other existing settings are preserved. Source bytes, catalog state, edits & ratings are never changed. CLI exits nonzero on interrupted/budget-stopped runs & checkpoints completed receipts before rendering their suggestions. Default admission budget is $0.50, sufficient for the three demo frames, five models & two repeats at frozen reserve rates.

Five-model shortlist is frozen against 9 October 2026 public capability/model-version metadata: DeepSeek V4.1 Flash, Gemini3.8 Flash, Gemini3.5 Flash-Lite, Qwen3.8 Flash & Haiku5.5. Metadata preflight requires image/structured-output support & matching canonical revision; endpoint routing requires strict schema parameters, no fallback, denied data collection & zero data retention. Lack of an eligible endpoint is a recorded failure, not permission to relax privacy or schema enforcement. API response records actual served model/provider, latency, usage/cost, errors & exact uploaded-byte hash. Canonical API metadata is evidence, not proof of an undisclosed provider weight revision.

No retries, tools, file uploads, streaming or durable response cache. At most twelve files, five models & three repeats. Requests cap completion tokens at2048, bodies at4MiB, responses at256KiB, & deadline at45s. OS DNS resolution can outlive request deadline; subsequent I/O checks it. Interrupted billing is unknown & stops further calls. Budget is an estimated admission reserve (20K input tokens + capped output at frozen upper rates); use provider-key spending limits for hard enforcement. Missing usage cost stops comparison. Provider price limits cap rates; billed totals are recorded without pretending to predict image-token billing exactly.

Model JSON must contain all fields & eight exact, finite, bounded controls for `adjust` with confidence≥0.7. Invalid JSON, unsupported controls, duplicate recipe fields, partial/refused completion & model substitution produce no recipe. `preserve`/`review` require explicit null recipe. Model confidence is uncalibrated; it is not an acceptance score. Input, current Auto & candidate renders are shown side by side; highlight/black clipping observations are recorded without penalizing intentional clipping automatically. No winner is selected without held-out shoots & blinded preference ratings.

## Qualification & next scope

Rust protocol, capability, no-mutation & HTTP regression tests run through generated RightKit GitHub Actions (`cargo xtask ci`). This is experimental CLI infrastructure, not installed desktop cloud Auto. Before an app workflow: integrate per-app secure credentials via compatible published RightKit Secrets, background jobs/cancel/progress, immutable engine-owned proposals & stale-check/one-undo apply; run hidden native QA against exact CI artifacts.

Shared owner integration: RightKit shell/control/QA already run desktop hosting. `rightkit-llm` has no verified published release here; `rightkit-http0.2.0` enables ureq's `rustls` feature, which enables `ring`. LightCraft's existing fetch transport uses rustls-rustcrypto; a small bounded JSON POST reuses it without new crypto/SDK dependencies. SDK files are unchanged. Revisit transport when published shared owner satisfies pure-Rust dependency policy.

Sources: [OpenRouter model catalog](https://openrouter.ai/api/v1/models), [structured outputs](https://openrouter.ai/docs/guides/features/structured-outputs), [provider routing/privacy](https://openrouter.ai/docs/guides/routing/provider-selection), [DeepSeek vision](https://api-docs.deepseek.com/guides/vision/).
