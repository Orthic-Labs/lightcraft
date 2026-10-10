# CI restoration record

Checked 2026-10-10 against `Orthic-Labs/lightcraft` at main revision
`2fef37a4c466acd8c0ae88a1ac5a1d0e1218dbc0`.

GitHub reports repository Actions enabled, all actions allowed, workflow
permissions `read`, and both `lightcraft-desktop-workspace-ci` and
`release-candidate` active. Repository metadata also reports `disabled:false`,
with current token permissions including `admin:true`. Supported restoration
calls were made once:

- `gh workflow enable ci.yml` → exit 0
- `gh workflow enable release-candidate.yml` → exit 0
- `PUT /repos/Orthic-Labs/lightcraft/actions/permissions` with
  `{"enabled":true,"allowed_actions":"all"}` → exit 0
- `PUT /repos/Orthic-Labs/lightcraft/actions/permissions/workflow` with
  `{"default_workflow_permissions":"read","can_approve_pull_request_reviews":false}` → exit 0

Dispatching generated `release-candidate.yml` for the current revision then
failed before a run was created:

```
HTTP 422: Actions has been disabled for this repository.
POST /repos/Orthic-Labs/lightcraft/actions/workflows/377917643/dispatches
```

The same repository state returns HTTP 409 for selected-actions settings
(`All actions and workflows are allowed on this repository`). Organization
Actions policy reads return HTTP 403: `You must be an org admin or have the
actions policies fine-grained permission` and require `admin:org`; this is an
access boundary, not proof that organization policy caused the failure.

Comparison with public Orthic repositories narrows scope. `pulse`, `legion`,
and `sellright` each report `disabled:false`, repository Actions `enabled:true`,
`allowed_actions:all`, active workflows, and recent runs (including
`pulse` run `38036036549`, `legion` run `38034999218`, and `sellright` run
`38030506274`). LightCraft alone returns the dispatch HTTP 422, so confirmed
cause is fork-usage suspension at GitHub Actions service layer. Public Actions
page [`Orthic-Labs/lightcraft/actions`](https://github.com/Orthic-Labs/lightcraft/actions)
shows: `Workflows are not currently being run on this fork due to the scale of
GitHub Actions usage. A repository maintainer can choose to re-enable these.`
This establishes fork suspension; it does not establish organization policy.

Last usable evidence predates current main: CI run `37870029202` passed at
`09fdad5e44b92dee301a272a8a9b31faf1ae15c4`; candidate run `37871082119` ran
both targets at that revision, with Windows passing and macOS hidden native QA
failing. No current-revision run exists because GitHub rejects dispatch before
workflow execution.

Workflow regeneration was inspected before retaining this correction. Current
`@rightkit/release` ownership source defines required hosts as `windows` and
`macos`; its trust-path selector throws `ownership approval unsupported
platform: linux`. Prior candidate admission on `macos-15` completed ownership
checks, while no Linux admission evidence exists. Template conformity therefore
cannot replace this runner until published RightKit source supports Linux.

GitHub’s [repository Actions settings guidance](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)
states that this banner indicates a separate GitHub-controlled disabled state;
changing repository settings does not restore access. One bounded reset attempt
preserved `allowed_actions:all`: `PUT .../actions/permissions` with
`{"enabled":false}` succeeded, then `{"enabled":true,"allowed_actions":"all"}`
succeeded. A first invalid false request including `allowed_actions` returned
HTTP 409 because that field is accepted only when enabled. One post-reset
dispatch still returned HTTP 422 `Actions has been disabled for this
repository`; no run was created.

Smallest remaining action: repository maintainer select public Actions page’s
supported `re-enable` control. GitHub documents this separate disabled state as
requiring Support review if settings changes do not restore access; repository
API settings cannot clear fork-usage suspension. Require completed
generated-workflow run for exact current SHA as CI evidence.

`pnpm exec right-git drift .` initially reported `release-candidate.yml`
mismatch. `pnpm exec right-git sync .` regenerated it, changing only admission
runner `macos-15` to template runner `ubuntu-24.04`. The supported macOS
correction is retained; resulting drift is intentional until template or SDK
adds Linux ownership approval.
