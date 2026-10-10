import { cpus, arch as hostArch, platform as hostPlatform, totalmem } from "node:os";
import { existsSync, readFileSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fail, safeRelative, sha256 } from "./lib.mjs";

const MAX_REPORT_BYTES = 4 * 1024 * 1024;
const TIMEOUT_MS = 180_000;
const HASH = /^[0-9a-f]{64}$/;
const REVISION = /^[0-9a-f]{40}$/;
const CASE_NAMES = ["flat-shadows", "color-patches-gradient", "hard-step-edge", "fine-texture", "intentional-soft-edge"];
const AMOUNTS = [0, 25, 50, 75, 100];

function hostInfo() {
  const model = cpus()[0]?.model?.trim();
  if (!model || Buffer.byteLength(model, "utf8") > 512 || [...model].some((value) => value === "/" || value === "\\" || value.charCodeAt(0) < 32)) {
    fail("denoise baseline requires a path-safe CPU model from the host OS API");
  }
  return {
    os: hostPlatform(),
    arch: hostArch(),
    cpuModel: model,
    cpuCount: cpus().length,
    memoryBytes: totalmem(),
  };
}

function targetHost(platform, architecture) {
  const expected = {
    macos: { host: "darwin", architectures: ["arm64"] },
    windows: { host: "win32", architectures: ["x86_64"] },
  }[platform];
  if (!expected || !expected.architectures.includes(architecture)) fail("denoise baseline target platform/architecture pair is unsupported");
  return { platform: expected.host, architecture: platform === "windows" ? "x64" : "arm64" };
}

function readReport(file) {
  if (!existsSync(file) || !statSync(file).isFile()) fail("denoise baseline report is missing: " + file);
  const size = statSync(file).size;
  if (size > MAX_REPORT_BYTES) fail("denoise baseline report exceeds 4 MiB cap");
  try {
    return JSON.parse(readFileSync(file, "utf8"));
  } catch (error) {
    fail("denoise baseline report is invalid JSON: " + error.message);
  }
}

function finite(value, label) {
  if (typeof value !== "number" || !Number.isFinite(value)) fail("denoise baseline " + label + " must be finite numeric data");
}

function hash(value, label) {
  if (typeof value !== "string" || !HASH.test(value)) fail("denoise baseline " + label + " must be SHA-256");
}

function validateReceipt(report, revision, hardware) {
  if (report.schema !== "lightcraft.denoise-baseline.v1" || report.status !== "alwaysUNQUALIFIED" || report.qualification !== "alwaysUNQUALIFIED" || report.mode !== "procedural-only") {
    fail("denoise baseline receipt qualification contract failed");
  }
  if (report.sourceRevision !== revision || report.hardware !== hardware || report.size !== 128 || report.sourceLong !== 128 || report.outputLong !== 128 || report.repeats !== 3 || report.warmRepeats !== 3 || report.firstCalls !== 1) {
    fail("denoise baseline invocation metadata does not match receipt");
  }
  if (!Array.isArray(report.cases) || report.cases.length !== 5) fail("denoise baseline must contain five procedural cases");
  for (const [caseIndex, item] of report.cases.entries()) {
    if (item.name !== CASE_NAMES[caseIndex]) fail("denoise baseline case order/name contract failed");
    hash(item.cleanSha256, "case " + caseIndex + " clean digest");
    hash(item.noisySha256, "case " + caseIndex + " noisy digest");
    hash(item.identity?.cleanSha256, "case " + caseIndex + " identity clean digest");
    hash(item.identity?.noisySha256, "case " + caseIndex + " identity noisy digest");
    hash(item.identity?.outputSha256, "case " + caseIndex + " identity output digest");
    if (item.identity?.source !== "identity" || item.identity?.cleanSha256 !== item.cleanSha256 || item.identity?.noisySha256 !== item.noisySha256 || item.identity?.outputSha256 !== item.noisySha256 || !Array.isArray(item.amounts) || item.amounts.length !== 5) {
      fail("denoise baseline case " + caseIndex + " identity/amount contract failed");
    }
    for (const [amountIndex, amount] of item.amounts.entries()) {
      if (amount.amount !== AMOUNTS[amountIndex] || amount.source !== "outputlong-maxdim") fail("denoise baseline amount/source contract failed");
      for (const key of ["cleanSha256", "noisySha256", "outputSha256"]) hash(amount[key], "case " + caseIndex + " amount " + amountIndex + " digest");
      if (amount.cleanSha256 !== item.cleanSha256 || amount.noisySha256 !== item.noisySha256) fail("denoise baseline amount input digest mismatch");
      const metrics = amount.metrics;
      for (const key of ["mse", "rmse", "gradientRmse"]) {
        finite(metrics?.[key], "case " + caseIndex + " amount " + amountIndex + " " + key);
        if (metrics[key] < 0) fail("denoise baseline metric must be nonnegative");
      }
      for (const value of metrics?.signedChannelBias ?? []) finite(value, "case " + caseIndex + " amount " + amountIndex + " channel bias");
      if (!Array.isArray(metrics?.signedChannelBias) || metrics.signedChannelBias.length !== 3) fail("case " + caseIndex + " amount " + amountIndex + " channel bias shape failed");
      if (metrics.psnrDbPeak1 !== null) finite(metrics.psnrDbPeak1, "case " + caseIndex + " amount " + amountIndex + " PSNR");
      for (const key of ["first", "warmP50", "warmP95"]) {
        finite(amount.timingMs?.[key], "case " + caseIndex + " amount " + amountIndex + " timing");
        if (amount.timingMs[key] < 0) fail("denoise baseline timing must be nonnegative");
      }
      if (amount.timingMs.warmCount !== 3 || amount.timingMs.warmP50 > amount.timingMs.warmP95) fail("denoise baseline warm timing contract failed");
      finite(amount.repeatMaxAbs, "case " + caseIndex + " amount " + amountIndex + " repeat maximum");
      finite(amount.repeatThreshold, "case " + caseIndex + " amount " + amountIndex + " repeat threshold");
      if (amount.repeatMaxAbs < 0 || amount.repeatThreshold < 0 || !Array.isArray(amount.repeatability?.errors) || amount.repeatability.errors.length !== 0) {
        fail("denoise baseline repeatability contract failed");
      }
      if (amount.repeatability.maxAbs !== amount.repeatMaxAbs || amount.repeatability.threshold !== amount.repeatThreshold || amount.repeatability.withinThreshold !== (amount.repeatMaxAbs <= amount.repeatThreshold)) {
        fail("denoise baseline repeatability measurement mismatch");
      }
      if (!Number.isSafeInteger(amount.validation?.outOfRangeChannelsAcrossRuns) || amount.validation.outOfRangeChannelsAcrossRuns < 0 || typeof amount.validation?.outputRangeWithinUnit !== "boolean") {
        fail("case " + caseIndex + " amount " + amountIndex + " output validation contract failed");
      }
      if (amount.validation.outputRangeWithinUnit !== (amount.validation.outOfRangeChannelsAcrossRuns === 0)) fail("denoise baseline range validation mismatch");
      if (amountIndex === 0 && (amount.outputSha256 !== item.noisySha256 || JSON.stringify(amount.metrics) !== JSON.stringify(item.identity.metrics))) fail("denoise baseline identity amount mismatch");
    }
  }
}

export function runBaseline({ cli, root, revision, platform, architecture }) {
  if (process.env.GITHUB_ACTIONS !== "true") fail("denoise baseline evidence requires generated GitHub Actions");
  if (!REVISION.test(revision)) fail("denoise baseline requires exact 40-character source revision");
  const target = targetHost(platform, architecture);
  if (process.platform !== target.platform || hostArch() !== target.architecture) fail("denoise baseline target does not match native runner");
  const hardware = hostInfo();
  const finalPath = safeRelative(root, "denoise-baseline.json", "denoise baseline report");
  const rawPath = safeRelative(root, ".denoise-baseline.raw.json", "denoise baseline temporary report");
  if (existsSync(finalPath) || existsSync(rawPath)) fail("denoise baseline report path already exists");
  const cliHash = sha256(cli);
  const result = spawnSync(cli, ["ai", "denoise", "baseline", "--hardware", hardware.cpuModel, "--source-revision", revision, "--out", rawPath, "--size", "128", "--repeats", "3"], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    timeout: TIMEOUT_MS,
    maxBuffer: 1024 * 1024,
  });
  if (result.error || result.status !== 0) fail("denoise baseline execution failed: " + (result.error?.message || result.stderr || result.status));
  const report = readReport(rawPath);
  validateReceipt(report, revision, hardware.cpuModel);
  if (sha256(cli) !== cliHash) fail("denoise baseline CLI changed during execution");
  const evidence = {
    schema: 1,
    sourceRevision: revision,
    platform,
    architecture,
    processingBackend: "cpu",
    cliSha256: cliHash,
    host: hardware,
  };
  report.ciEvidence = evidence;
  const encoded = JSON.stringify(report, null, 2) + "\n";
  if (Buffer.byteLength(encoded, "utf8") > MAX_REPORT_BYTES) fail("denoise baseline evidence report exceeds 4 MiB cap");
  try {
    writeFileSync(finalPath, encoded, { encoding: "utf8", flag: "wx" });
  } finally {
    try { unlinkSync(rawPath); } catch {}
  }
  console.log("[candidate] denoise baseline retained: " + path.relative(root, finalPath) + " cli=" + evidence.cliSha256 + " status=alwaysUNQUALIFIED");
  return finalPath;
}
