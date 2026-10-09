# Right-suite photo editor fork

Keep a product fork for RightKit React/Tauri UX, optional cloud provider experiments, local culling & Auto work. Contribute broadly useful RAW/catalog/processing fixes upstream separately. Upstream remains engine/reference source; our release gate owns shipped behavior.

## Sync policy

- Fetch `origin/main` into an upstream tracking branch; record inspected commit & merged PR IDs.
- Bring security, crash/data-loss, decoder, color & catalog compatibility fixes into an integration branch first. Read associated tests & supporting commits; preserve authorship & record source SHA. Keep schema migrations with their readers/writers, not isolated hunks.
- Prefer whole commits/PRs. If engine changes need dependencies, bring related chain together. Keep RightKit/UI/cloud changes separate so conflict decisions remain visible.
- Run generated RightKit CI & exact-artifact hidden Mac/Windows qualification before promotion. Fork patches remain explicit; compare upstream test coverage before marking issues absorbed.
- Tag qualified releases & retain rollback artifacts. A clean source merge is not a qualified release.

## Branding

Working name: **PhotoRight**, matching ScrapeRight/CutRight naming. Prepare an original Right-suite mark; retain LightCraft/ArtCraft copyrights, licences, NOTICE & contributor credits in About/source. Do not derive our mark from existing ArtCraft assets. Their [brand licence](../docs/brand/LICENSE-brand.txt) requires distributed product forks to replace ArtCraft marks & permits plain-text upstream attribution. No product/bundle ID, storage directory or desktop registration is renamed by this experiment; branding rollout needs explicit migration design to preserve catalogs/settings & avoid duplicate app registrations.

## RightKit integrations

Already integrated: shell, React shell/platform/theme, native control, hidden QA & generated Git/release workflows. Useful next owners: per-app secrets, typed settings, job lifecycle/progress & model acquisition where published contracts fit.

Replace infrastructure only after a source/API/dependency/qualification comparison. Keep domain-specific Rust RAW decoding, color, rendering, catalog semantics & validated custom SQLite importer. A RightKit facade over an external runtime does not remove that runtime's dependencies or licence/distribution burden. No shared SDK edits are part of this app task.

Photo assessment currently reuses existing pure-Rust fetch crypto; published RightKit HTTP0.2.0 enables `ring`. This is an explicit compatibility gap, not an invitation to fork an entire provider SDK. `lightcraft-photo-ai` contains only bounded protocol/validation & transport composition; schema, prompt & model fixtures are shared by command hosts.
