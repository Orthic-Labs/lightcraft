import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
export default defineConfig({ root: fileURLToPath(new URL('.', import.meta.url)), plugins: [react()], clearScreen: false, server: { port: 18483, strictPort: true, host: '127.0.0.1' }, build: { target: ['es2022'], outDir: 'dist', emptyOutDir: true } });
