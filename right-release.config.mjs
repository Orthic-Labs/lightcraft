import path from 'node:path';
const build = { cmd: 'node', args: ['scripts/desktop/build-native.mjs'] };
export default {
  schema: 1,
  app: 'lightcraft-desktop',
  packageManager: 'pnpm@11.24.0',
  development: {
    targets: {
      mac: { build: [build], install: { kind: 'tauri', productName: 'Ember', targetTriple: 'aarch64-apple-darwin', manifestPath: 'apps/lightcraft-desktop/Cargo.toml', destination: '/Applications/Ember.app' } },
      win: { build: [build], install: { kind: 'tauri', productName: 'Ember', targetTriple: 'x86_64-pc-windows-msvc', manifestPath: 'apps/lightcraft-desktop/Cargo.toml', expectedInstalledPath: path.join(process.env.LOCALAPPDATA || '.', 'Ember', 'lightcraft-desktop.exe') } },
    },
  },
};
