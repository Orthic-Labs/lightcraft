import { appendFileSync, existsSync, mkdirSync, readFileSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runCargoSync } from '@rightkit/release/managed-cargo.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const configPath = path.join(root, '.upstream-pr-qualification.json');
const worktree = path.join(process.env.RUNNER_TEMP || os.tmpdir(), `lightcraft-pr-434-${process.pid}`);
const statuses = [];

function fail(message) {
  throw new Error(message);
}

function git(args, cwd = root) {
  const result = spawnSync('git', args, { cwd, env: process.env, encoding: 'utf8', windowsHide: true });
  if (result.error || result.status !== 0) fail(`git ${args[0]} failed: ${result.error?.message || result.stderr?.trim() || result.status}`);
  return result.stdout.trim();
}

function cargo(label, args, cwd) {
  console.log(`[upstream PR 434] ${label}`);
  const env = { ...process.env, CARGO_TARGET_DIR: path.join(root, 'target'), RUST_TEST_THREADS: '2' };
  const result = runCargoSync(args, { cwd, env, stdio: 'inherit', windowsHide: true });
  const status = result.error ? `error: ${result.error.message}` : result.status === 0 ? 'passed' : `failed (${result.status ?? result.signal ?? 'unknown'})`;
  statuses.push({ label, status });
  console.log(`[upstream PR 434] ${label}: ${status}`);
}

function appendSummary(config, setupError = null) {
  const summaryFile = process.env.GITHUB_STEP_SUMMARY;
  if (!summaryFile) return;
  const rows = [
    '',
    '### Upstream PR 434 CI',
    '',
    `- Source: \`${config?.sourceRevision || 'unavailable'}\``,
    `- Base: \`${config?.baseRevision || 'unavailable'}\``,
    ...statuses.map(({ label, status }) => `- ${label}: **${status}**`),
  ];
  if (setupError) rows.push(`- Setup: **failed** (${setupError})`);
  appendFileSync(summaryFile, `${rows.join('\n')}\n`, 'utf8');
}

async function main() {
  if (process.env.GITHUB_ACTIONS !== 'true') fail('upstream PR gate requires GitHub Actions');
  let config;
  try {
    config = JSON.parse(readFileSync(configPath, 'utf8'));
    if (config.repository !== 'Orthic-Labs/lightcraft' || config.pr !== 434 || !/^[0-9a-f]{40}$/.test(config.sourceRevision) || !/^[0-9a-f]{40}$/.test(config.baseRevision)) {
      fail('invalid .upstream-pr-qualification.json; expected repository, PR 434, and pinned 40-character revisions');
    }
    const object = spawnSync('git', ['cat-file', '-e', `${config.sourceRevision}^{commit}`], { cwd: root, env: process.env, windowsHide: true });
    if (object.error || object.status !== 0) git(['fetch', '--no-tags', 'origin', config.sourceRevision]);
    git(['cat-file', '-e', `${config.sourceRevision}^{commit}`]);
    const baseObject = spawnSync('git', ['cat-file', '-e', `${config.baseRevision}^{commit}`], { cwd: root, env: process.env, windowsHide: true });
    if (baseObject.error || baseObject.status !== 0) git(['fetch', '--no-tags', 'origin', config.baseRevision]);
    const ancestry = spawnSync('git', ['merge-base', '--is-ancestor', config.baseRevision, config.sourceRevision], { cwd: root, env: process.env, windowsHide: true });
    if (ancestry.error || ancestry.status !== 0) fail(`pinned base ${config.baseRevision} is not an ancestor of source ${config.sourceRevision}`);
    mkdirSync(path.dirname(worktree), { recursive: true });
    git(['worktree', 'add', '--detach', worktree, config.sourceRevision]);
    const actual = git(['rev-parse', 'HEAD'], worktree);
    if (actual !== config.sourceRevision) fail(`worktree revision mismatch: expected ${config.sourceRevision}, got ${actual}`);
    if (git(['status', '--porcelain'], worktree) !== '') fail('source worktree is not clean');

    const cwd = worktree;
    cargo('engine XMP tests', ['test', '--locked', '-p', 'lightcraft-engine', 'tests_xmp'], cwd);
    cargo('engine import tests', ['test', '--locked', '-p', 'lightcraft-engine', 'tests_import'], cwd);
    cargo('UI locale/build script check', ['check', '--locked', '-p', 'lightcraft-ui-egui'], cwd);
    cargo('full cargo xtask ci', ['run', '--locked', '-p', 'xtask', '--', 'ci'], cwd);
    console.log(`[upstream PR 434] full CI source SHA: ${config.sourceRevision}`);
    const clean = git(['status', '--porcelain'], cwd) === '';
    statuses.push({ label: 'source worktree clean after full CI', status: clean ? 'passed' : 'failed (working tree changed)' });
  } catch (error) {
    appendSummary(config, error.message);
    throw error;
  } finally {
    if (existsSync(worktree)) {
      const result = spawnSync('git', ['worktree', 'remove', '--force', worktree], { cwd: root, env: process.env, stdio: 'inherit', windowsHide: true });
      if (result.error || result.status !== 0) rmSync(worktree, { recursive: true, force: true });
    }
  }
  appendSummary(config);
  const failed = statuses.filter(({ status }) => status !== 'passed');
  if (failed.length) fail(`${failed.length} upstream CI gate(s) failed`);
  console.log('[upstream PR 434] PASS');
}

main().catch((error) => {
  console.error(`[upstream PR 434] ${error.message}`);
  process.exitCode = 1;
});
