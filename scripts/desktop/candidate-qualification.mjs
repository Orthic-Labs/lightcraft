import { runDevelopment } from '@rightkit/release/development.mjs';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { runCargoSync } from '@rightkit/release/managed-cargo.mjs';
import { fail, loadReleaseConfig, repoRoot, safeRelative } from './lib.mjs';
import path from 'node:path';
export async function qualify(record, root) {
  if (process.env.GITHUB_ACTIONS !== 'true') fail('candidate qualification requires generated Actions');
  const executable = record.artifacts.find(artifact => artifact.target === 'lightcraft-desktop' && artifact.kind === 'executable');
  if (!executable) fail('candidate lacks LightCraft desktop executable');
  const { config } = await loadReleaseConfig();
  const targetName = record.platform === 'macos' ? 'mac' : 'win';
  const target = config.development.targets[targetName];
  const installedConfig = { ...config, development: { targets: { [targetName]: { ...target, build: [{ cmd: 'node', args: ['scripts/desktop/verify-built.mjs'] }] } } } };
  runDevelopment({ root: repoRoot, platform: targetName, config: installedConfig });
  const installed = record.platform === 'macos' ? path.join(target.install.destination, 'Contents/MacOS/lightcraft-desktop') : target.install.expectedInstalledPath;
  const installedHash = createHash('sha256').update(readFileSync(installed)).digest('hex');
  const env = { ...process.env, RIGHTKIT_QA_HIDDEN: '1', RIGHTKIT_QA_UI_BINARY: installed, RIGHTKIT_QA_EVIDENCE: path.join(root, 'qa-evidence'), RIGHTKIT_QA_SOURCE_REVISION: record.sourceRevision, RIGHTKIT_QA_ARCHITECTURE: record.architecture, RIGHTKIT_QA_INSTALLED_ARTIFACT_SHA256: installedHash };
  const result = runCargoSync(['test', '--locked', '-p', 'lightcraft-desktop', '--test', 'native_qualification', '--features', 'qa-native', '--', '--ignored', '--nocapture'], { cwd: repoRoot, env, stdio: 'inherit', windowsHide: true });
  if (result.error || result.status !== 0) fail(`native qualification failed: ${result.error?.message || result.status}`);
}
