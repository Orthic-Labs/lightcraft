import path from 'node:path';
const build = { cmd: 'node', args: ['scripts/desktop/build-native.mjs'] };
export default {
  schema: 1,
  app: 'lightcraft-desktop',
  packageManager: 'pnpm@11.24.0',
  development: {
    targets: {
      mac: { build: [build], install: { kind: 'tauri', productName: 'LightCraft Preview', targetTriple: 'aarch64-apple-darwin', manifestPath: 'apps/lightcraft-desktop/Cargo.toml', destination: '/Applications/LightCraft Preview.app' } },
      win: { build: [build], install: { kind: 'tauri', productName: 'LightCraft Preview', targetTriple: 'x86_64-pc-windows-msvc', manifestPath: 'apps/lightcraft-desktop/Cargo.toml', expectedInstalledPath: path.join(process.env.LOCALAPPDATA || '.', 'LightCraft Preview', 'lightcraft-desktop.exe') } },
    },
  },
};
