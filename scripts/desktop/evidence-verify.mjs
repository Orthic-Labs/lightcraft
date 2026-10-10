import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { currentPlatform, fail, repoRoot, safeRelative, sha256, sourceIdentity } from "./lib.mjs";

if (process.env.GITHUB_ACTIONS !== "true") fail("release evidence verification is GitHub Actions-only");
const revision = sourceIdentity();
const version = process.env.RIGHT_GIT_RELEASE_VERSION;
const runId = process.env.RIGHT_GIT_RUN_ID;
const runAttempt = process.env.RIGHT_GIT_RUN_ATTEMPT;
if (!version || !runId || !runAttempt) fail("release evidence requires version, run id, & run attempt");
const summaryRootName = process.env.RIGHT_GIT_STAGE_SUMMARY_ROOT;
const evidenceRootName = process.env.RIGHT_GIT_QUALIFICATION_EVIDENCE_ROOT;
if (!summaryRootName || !evidenceRootName) fail("release evidence roots are required");
const summaryRoot = path.resolve(summaryRootName);
const evidenceRoot = path.resolve(evidenceRootName);

function readSummary(directory) {
  const file = path.join(summaryRoot, directory, "stage-summary.json");
  let value;
  try { value = JSON.parse(readFileSync(file, "utf8")); } catch (error) { fail(`missing or invalid stage summary ${directory}: ${error.message}`); }
  for (const [key, expected] of [["schema", 1], ["status", "SUCCEEDED"], ["version", version], ["sourceRevision", revision], ["runId", runId], ["runAttempt", runAttempt]]) {
    if (value[key] !== expected) fail(`stage summary ${directory} identity mismatch at ${key}`);
  }
  return value;
}

const summaries = ["candidate-windows", "windows-sign", "installed-qualification"];
if (process.env.RIGHT_GIT_FINALIZED_MACOS_ROOT) summaries.push("candidate-macos", "macos-sign");
for (const directory of summaries) readSummary(directory);

function files(root) {
  const output = [];
  function walk(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(file);
      else if (entry.isFile()) output.push(file);
    }
  }
  walk(root);
  return output;
}
const evidenceFiles = files(evidenceRoot);
const bundles = evidenceFiles.filter((file) => path.basename(file) === "evidence.json");
if (!evidenceFiles.some((file) => path.basename(file) === "summary.jsonl") || bundles.length === 0) fail("RightKit QA summary.jsonl & evidence.json are required");
let rollback = false;
for (const file of bundles) {
  let bundle;
  try { bundle = JSON.parse(readFileSync(file, "utf8")); } catch (error) { fail(`invalid RightKit QA bundle ${file}: ${error.message}`); }
  if (bundle.status !== "passed") fail(`RightKit QA bundle did not pass: ${file}`);
  if (bundle.identity?.sourceRevision !== revision) fail(`RightKit QA bundle source identity mismatch: ${file}`);
  for (const check of bundle.checks ?? []) if (check.status !== "passed") fail(`RightKit QA check did not pass: ${file}`);
  if (bundle.identity?.scenarios?.includes("rollback")) rollback = true;
}
if (!rollback) fail("RightKit QA rollback evidence is missing");

const finalized = path.resolve(process.env.RIGHT_GIT_FINALIZED_WINDOWS_ROOT || "");
if (!process.env.RIGHT_GIT_FINALIZED_WINDOWS_ROOT) fail("finalized Windows root is required for release evidence");
const recordFile = path.join(finalized, "compiler-artifacts.json");
let record;
try { record = JSON.parse(readFileSync(recordFile, "utf8")); } catch (error) { fail(`cannot read finalized compiler artifact record: ${error.message}`); }
const platform = currentPlatform();
if (record.schema !== 1 || record.sourceRevision !== revision || record.platform !== platform) fail("finalized compiler artifact identity mismatch");
for (const artifact of record.artifacts ?? []) {
  const file = safeRelative(finalized, artifact.path, "finalized artifact");
  if (sha256(file) !== artifact.sha256) fail(`finalized artifact hash mismatch: ${artifact.path}`);
}
console.log(`desktop release evidence: PASS ${version} ${revision} (${bundles.length} QA bundles)`);
