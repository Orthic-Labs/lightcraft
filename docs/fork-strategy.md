# Ember fork strategy

Keep a product fork for RightKit React/Tauri UX, optional cloud provider experiments, local culling & Auto work. Contribute broadly useful RAW/catalog/processing fixes upstream separately. Upstream remains engine/reference source; our release gate owns shipped behavior.

## Sync policy

- Fetch `origin/main` into an upstream tracking branch; record inspected commit & merged PR IDs.
- Bring security, crash/data-loss, decoder, color & catalog compatibility fixes into an integration branch first. Read associated tests & supporting commits; preserve authorship & record source SHA. Keep schema migrations with their readers/writers, not isolated hunks.
- Prefer whole commits/PRs. If engine changes need dependencies, bring related chain together. Keep RightKit/UI/cloud changes separate so conflict decisions remain visible.
- Run generated RightKit CI & exact-artifact hidden Mac/Windows qualification before promotion. Fork patches remain explicit; compare upstream test coverage before marking issues absorbed.
- Tag qualified releases & retain rollback artifacts. A clean source merge is not a qualified release.

## Branding

Product name: **Ember**, maintained by Orthic Labs. Original flame artwork replaces upstream product marks. Preserve LightCraft/ArtCraft copyrights, licences, NOTICE & contributor credits. Plain-text upstream attribution appears in About & README. See [branding & compatibility](branding.md) for bundle identity, legacy preferences/library reuse & retained format identifiers.

## RightKit integrations

Already integrated: shell, React shell/platform/theme, native control, hidden QA & generated Git/release workflows. Useful next owners: per-app secrets, typed settings, job lifecycle/progress & model acquisition where published contracts fit.

Replace infrastructure only after a source/API/dependency/qualification comparison. Keep domain-specific Rust RAW decoding, color, rendering, catalog semantics & validated custom SQLite importer. A RightKit facade over an external runtime does not remove that runtime's dependencies or licence/distribution burden. No shared SDK edits are part of this app task.

Photo assessment currently reuses existing pure-Rust fetch crypto; published RightKit HTTP0.2.0 enables `ring`. This is an explicit compatibility gap, not an invitation to fork an entire provider SDK. `lightcraft-photo-ai` contains only bounded protocol/validation & transport composition; schema, prompt & model fixtures are shared by command hosts.
