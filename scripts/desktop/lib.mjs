import { createHash } from "node:crypto";
import { appendFileSync, copyFileSync, existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

export function fail(message) {
  throw new Error(message);
}

export function readJson(file) {
  const resolved = path.resolve(repoRoot, file);
  try {
    return JSON.parse(readFileSync(resolved, "utf8"));
  } catch (error) {
    fail(`cannot read JSON ${path.relative(repoRoot, resolved)}: ${error.message}`);
  }
}

export function sha256(file) {
  return createHash("sha256").update(readFileSync(file)).digest("hex");
}

export function requireFile(file, label = file) {
  if (!existsSync(file) || !statSync(file).isFile()) fail(`${label} is missing: ${file}`);
  return file;
}

export function requireEnv(name) {
  const value = process.env[name];
  if (!value || !value.trim()) fail(`missing required environment variable ${name}`);
  return value;
}

export function safeRelative(root, candidate, label) {
  const resolvedRoot = path.resolve(root);
  const resolved = path.resolve(root, candidate);
  const relative = path.relative(resolvedRoot, resolved);
  if (!relative || relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    fail(`${label} must stay inside ${resolvedRoot}`);
  }
  return resolved;
}

export function commandParts(value, label) {
  if (Array.isArray(value) && value.length > 0 && value.every((part) => typeof part === "string" && part.length > 0)) return value;
  if (value && typeof value === "object" && typeof value.cmd === "string" && value.cmd.length > 0 && (!value.args || Array.isArray(value.args))) {
    return [value.cmd, ...(value.args ?? [])];
  }
  fail(`${label} must be a non-empty command array or {cmd,args}`);
}

export function commandHasPublication(parts) {
  return parts.some((part) => /(^|[-_:\s])(publish|upload)([-_:\s]|$)/i.test(part));
}

export function outputPath(name) {
  const file = process.env.GITHUB_OUTPUT;
  if (!file) return null;
  return file;
}

export function writeOutput(values) {
  const file = outputPath();
  if (!file) return;
  const lines = Object.entries(values).map(([key, value]) => `${key}=${String(value)}`);
  appendFileSync(file, `${lines.join("\n")}\n`, "utf8");
}

export function currentPlatform() {
  const platform = process.env.RIGHT_GIT_RELEASE_PLATFORM;
  if (platform === "macos" || platform === "windows") return platform;
  if (process.env.RIGHT_GIT_FINALIZED_WINDOWS_ROOT) return "windows";
  if (process.env.RIGHT_GIT_FINALIZED_MACOS_ROOT) return "macos";
  if (process.platform === "darwin") return "macos";
  if (process.platform === "win32") return "windows";
  fail("native desktop script requires macOS or Windows runner");
}

export function releaseConfigPath() {
  return path.resolve(repoRoot, process.env.RIGHTKIT_RELEASE_CONFIG || "right-release.config.mjs");
}

export async function loadReleaseConfig() {
  const configPath = releaseConfigPath();
  requireFile(configPath, "RightKit release config");
  const module = await import(`${pathToFileURL(configPath).href}?desktop=${Date.now()}`);
  if (!module.default || typeof module.default !== "object") fail("RightKit release config must export default object");
  return { config: module.default, configPath };
}

export function artifactRoot() {
  return requireEnv("RIGHT_GIT_ARTIFACT_ROOT");
}

export function finalizedRoot() {
  const platform = currentPlatform();
  return requireEnv(platform === "macos" ? "RIGHT_GIT_FINALIZED_MACOS_ROOT" : "RIGHT_GIT_FINALIZED_WINDOWS_ROOT");
}

export function sourceIdentity() {
  const revision = requireEnv("RIGHT_GIT_SOURCE_REVISION");
  if (!/^[0-9a-f]{40}$/.test(revision)) fail("RIGHT_GIT_SOURCE_REVISION must be 40 lowercase hexadecimal characters");
  return revision;
}

/** Cargo's --message-format=json output is newline-delimited. Keep only real
 * compiler artifacts & resolve every filename against record location. */
export function cargoCompilerArtifacts(recordFile) {
  requireFile(recordFile, "Cargo compiler artifact record");
  const text = readFileSync(recordFile, "utf8");
  const rows = [];
  try {
    const value = JSON.parse(text);
    if (Array.isArray(value)) rows.push(...value);
    else rows.push(value);
  } catch {
    for (const line of text.split(/\r?\n/).map((value) => value.trim()).filter(Boolean)) {
      try { rows.push(JSON.parse(line)); } catch (error) { fail(`invalid Cargo JSON record line: ${error.message}`); }
    }
  }
  const base = path.dirname(path.resolve(recordFile));
  const output = [];
  const add = (file, target, kind) => {
    if (typeof file !== "string" || !file.trim()) return;
    const resolved = path.resolve(base, file);
    if (!output.some((entry) => entry.path === resolved)) output.push({ path: resolved, target, kind });
  };
  for (const row of rows) {
    if (row?.reason === "compiler-artifact") {
      const target = row.target?.name ?? "unknown";
      const kinds = Array.isArray(row.target?.kind) ? row.target.kind : [];
      if (typeof row.executable === "string") add(row.executable, target, kinds.includes("bin") ? "executable" : kinds.join(","));
      for (const file of row.filenames ?? []) {
        if (typeof file === "string" && /(?:^|[\\/])(?:[^/\\]+)\.(?:exe|app|dylib|dll|so)$/i.test(file)) add(file, target, kinds.join(","));
      }
    }
    if (row?.schema === 1 && Array.isArray(row.artifacts)) {
      for (const artifact of row.artifacts) add(artifact?.path, artifact?.target ?? "unknown", artifact?.kind ?? "artifact");
    }
  }
  if (output.length === 0) fail(`Cargo compiler artifact record has no executable artifacts: ${recordFile}`);
  return output;
}

export function materializeArtifacts(entries, destination, identity) {
  mkdirSync(destination, { recursive: true });
  const artifacts = entries.map((entry, index) => {
    requireFile(entry.path, `Cargo artifact ${entry.path}`);
    const name = path.basename(entry.path);
    const target = path.join(destination, `${String(index + 1).padStart(2, "0")}-${name}`);
    copyFileSync(entry.path, target);
    return { path: path.relative(destination, target).replaceAll("\\", "/"), sha256: sha256(target), kind: entry.kind, target: entry.target };
  });
  const record = { schema: 1, ...identity, artifacts };
  writeFileSync(path.join(destination, "compiler-artifacts.json"), `${JSON.stringify(record, null, 2)}\n`, "utf8");
  return { record, file: path.join(destination, "compiler-artifacts.json") };
}
