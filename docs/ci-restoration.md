# CI restoration record

## Restoration confirmed, 2026-10-10

Maintainer selected GitHub's fork `re-enable` control. Dispatching existing
`Packaging lint` workflow on main succeeded & created
[run 38053737558](https://github.com/Orthic-Labs/lightcraft/actions/runs/38053737558)
for source revision `561a076eed0d5bb506de3cdf09048ccd435ad8e2`.
This confirms workflow execution is available again; validation results still
require completed runs bound to their exact source revision.

Full workspace CI runs on the next main push. Unsigned candidate qualification
then uses that exact revision on macOS & Windows, with signing/publication
disabled. Published RightKit foreground readers still require repair before
macOS background evidence can qualify. Repair proposal is isolated outside
shared SDK checkout; no SDK changes or dependency upgrades have been applied.

## RightKit ownership handoff, 2026-10-10

Adrian assigned SDK work to Claude chat **Rightkit Mac new** & instructed Ember
chat to leave RightKit work there. Repair request, isolated reviewed patch,
native failure evidence & required qualification were sent through Pulse to
`Rightkit Mac new on Adrian’s Mac Mini` (message
`56d6f028-baa9-4e64-bbab-ff8ba4403484`). Corrected full proposal base SHA was
sent separately. Pulse reported `sent`
without recipient verdict; acknowledgment is pending. Do not resend or poll
its inbox. No SDK files, versions or packages were changed by Ember chat.

SDK owner will handle repair, native feature-matrix qualification & managed
publication. Ember will consume qualified published versions, then rerun
exact-revision hidden native QA with existing hard foreground guard intact.

## Suspension history

Checked 2026-10-10 against `Orthic-Labs/lightcraft` at main revision
`2fef37a4c466acd8c0ae88a1ac5a1d0e1218dbc0`.

Rechecked after source snapshot `3036f57cd1ee7bae8fd0183d29be34ced46335a9` was pushed to fork main: exact-revision run listing is empty & refreshed Actions page still displays same fork-usage suspension banner. Hidden browser is signed out; no maintainer re-enable control is available in current session. No restoration calls were repeated during this check.

Rechecked source snapshot `26100c66092503d4dca440e99f2903f863304617`: exact-revision Actions API returns zero runs. Available in-app browser remains signed out & public Actions page still shows fork-usage suspension. No alternate connected authenticated browser is available; no settings reset or dispatch was repeated.

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

Mac candidate job `113631654261` failed its hard foreground guard: sampled names were `LSDisplayName`,
`owned_frontmost_pids` was empty & PID sampling was unavailable. This is missing proof of hidden operation, not proof
that an owned process activated. Native IPC/scalability journeys also reported stale hidden WebKit layout. Current
source retains foreground assertions & does not launch/install another app to work around these failures.

Registry-source audit of `rightkit-qa` 0.2.12 versus 0.2.14 found identical control/process/guard implementations.
0.2.14 changes ffprobe precedence, hashing provider & dependency edges; it does not repair unavailable frontmost PID
sampling. Exact 0.2.12 pin is retained. Qualification requires reliable PID evidence & passing layout journeys, not
relaxing `owned_never_frontmost` when observed owned PID list is empty.

`rightkit-control` 0.1.9/0.1.10/0.1.11 have identical `mac.rs` frontmost readers. Separate `lsappinfo` calls parse an
exact `pid =` substring for PID & the first quoted token for name; missing PID/inventory evidence fails closed. Name
`LSDisplayName` is therefore not reliable application identity. QA StopReport does not distinguish unavailable PID,
inventory failure & missing identity registration. Required SDK repair is robust PID acquisition plus explicit failure
diagnostics, then published dependency update & exact-artifact native qualification. No published upgrade solves it.

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
