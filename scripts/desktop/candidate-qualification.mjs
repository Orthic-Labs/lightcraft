import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { runCargoSync } from '@rightkit/release/managed-cargo.mjs';
import { fail, loadReleaseConfig, repoRoot } from './lib.mjs';
import path from 'node:path';
export async function qualify(record, root) {
  if (process.env.GITHUB_ACTIONS !== 'true') fail('candidate qualification requires generated Actions');
  const executable = record.artifacts.find(artifact => artifact.target === 'lightcraft-desktop' && artifact.kind === 'executable');
  if (!executable) fail('candidate lacks LightCraft desktop executable');
  const { config } = await loadReleaseConfig();
  const targetName = record.platform === 'macos' ? 'mac' : 'win';
  const target = config.development.targets[targetName];
  const installed = record.platform === 'macos' ? path.join(target.install.destination, 'Contents/MacOS/lightcraft-desktop') : target.install.expectedInstalledPath;
  const installation = JSON.parse(readFileSync(path.join(root, 'installed-candidate.json'), 'utf8'));
  if (installation.schema !== 1 || installation.sourceRevision !== record.sourceRevision || installation.platform !== record.platform || installation.architecture !== record.architecture || installation.path !== installed) fail('installed candidate identity does not match qualified source & target');
  const installedHash = createHash('sha256').update(readFileSync(installed)).digest('hex');
  if (installation.sha256 !== installedHash) fail('installed candidate changed after Right Release installation');
  const env = { ...process.env, CRAFT_FONTS_DIR: path.join(process.env.RUNNER_TEMP, 'lightcraft-build-fonts'), CRAFT_FONTS_REQUIRED: '1', RIGHTKIT_QA_HIDDEN: '1', RIGHTKIT_QA_UI_BINARY: installed, RIGHTKIT_QA_EVIDENCE: path.join(root, 'qa-evidence'), RIGHTKIT_QA_SOURCE_REVISION: record.sourceRevision, RIGHTKIT_QA_ARCHITECTURE: record.architecture, RIGHTKIT_QA_INSTALLED_ARTIFACT_SHA256: installedHash };
  // Match candidate target/profile/features so qualification reuses native engine
  // compilation instead of rebuilding it under Cargo's default debug profile.
  const result = runCargoSync(['test', '--locked', '-p', 'lightcraft-desktop', '--test', 'native_qualification', '--target', target.install.targetTriple, '--profile', 'release-iterate', '--features', 'qa-native,custom-protocol', '--', '--ignored', '--nocapture'], { cwd: repoRoot, env, stdio: 'inherit', windowsHide: true });
  if (result.error || result.status !== 0) fail(`native qualification failed: ${result.error?.message || result.status}`);
  if (record.platform === 'macos') {
    // GitHub directory artifacts lose Unix modes. Retain the exact installed,
    // ad-hoc signed bundle exercised above in Apple's resource-preserving ZIP.
    const archive = path.join(root, 'packages', 'LightCraft-Preview-macos-arm64-qualified.zip');
    mkdirSync(path.dirname(archive), { recursive: true });
    const packed = spawnSync('/usr/bin/ditto', ['-c', '-k', '--sequesterRsrc', '--keepParent', target.install.destination, archive], { stdio: 'inherit' });
    if (packed.error || packed.status !== 0) fail(`qualified Mac bundle archive failed: ${packed.error?.message || packed.status}`);
    const archiveHash = createHash('sha256').update(readFileSync(archive)).digest('hex');
    writeFileSync(path.join(root, 'qualified-bundle.json'), JSON.stringify({ schema: 1, sourceRevision: record.sourceRevision, platform: record.platform, architecture: record.architecture, path: path.relative(root, archive), sha256: archiveHash, installedExecutableSha256: installedHash, nativeQualification: 'passed' }, null, 2) + '\n');
  }
}
