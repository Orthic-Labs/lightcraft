import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fail, requireEnv, safeRelative, sha256, sourceIdentity } from "./lib.mjs";

const action = requireEnv("RIGHT_GIT_STAGE_ACTION");
if (!new Set(["init", "finalize"]).has(action)) fail("RIGHT_GIT_STAGE_ACTION must be init or finalize");
const root = path.resolve(requireEnv("RIGHT_GIT_STAGE_ROOT"));
const evidenceRoot = path.resolve(requireEnv("RIGHT_GIT_STAGE_EVIDENCE_ROOT"));
const stage = requireEnv("RIGHT_GIT_STAGE");
const producer = requireEnv("RIGHT_GIT_STAGE_PRODUCER");
const revision = sourceIdentity();
const platform = requireEnv("RIGHT_GIT_RELEASE_PLATFORM");
const architecture = requireEnv("RIGHT_GIT_RELEASE_ARCHITECTURE");
const summaryFile = path.join(root, "stage-summary.json");
mkdirSync(root, { recursive: true });

function evidenceFiles() {
  if (!existsSync(evidenceRoot)) return [];
  let names = [];
  try { names = readdirSync(evidenceRoot, { recursive: true, withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => path.join(entry.parentPath ?? evidenceRoot, entry.name)); } catch (error) { fail(`cannot inspect stage evidence root: ${error.message}`); }
  return names.filter((file) => path.basename(file) !== "stage-summary.json").map((file) => {
    const relative = path.relative(evidenceRoot, file).replaceAll("\\", "/");
    safeRelative(evidenceRoot, relative, "stage evidence");
    return { name: relative, sha256: sha256(file) };
  }).sort((a, b) => a.name.localeCompare(b.name));
}

const previous = action === "finalize" && summaryFile ? (() => { try { return JSON.parse(readFileSync(summaryFile, "utf8")); } catch { return null; } })() : null;
if (action === "finalize" && !previous) fail("stage summary finalize requires init output");
const status = action === "finalize" ? (process.env.RIGHT_GIT_STAGE_STATUS || "FAILED") : "RUNNING";
const exitCode = action === "finalize" ? Number(process.env.RIGHT_GIT_STAGE_EXIT_CODE ?? 1) : null;
if (action === "finalize" && status === "SUCCEEDED" && exitCode !== 0) fail("successful stage must have exit code 0");
if (action === "finalize" && status !== "SUCCEEDED" && exitCode === 0) fail("failed stage cannot have exit code 0");
const summary = {
  schema: 1,
  stage,
  producer,
  action,
  status,
  exitCode,
  version: requireEnv("RIGHT_GIT_RELEASE_VERSION"),
  sourceRevision: revision,
  platform,
  architecture,
  runId: requireEnv("RIGHT_GIT_RUN_ID"),
  runAttempt: requireEnv("RIGHT_GIT_RUN_ATTEMPT"),
  evidence: evidenceFiles(),
  previousAction: previous?.action ?? null,
};
writeFileSync(summaryFile, `${JSON.stringify(summary, null, 2)}\n`, "utf8");
console.log(`desktop stage summary: ${action} ${stage} ${status}`);
