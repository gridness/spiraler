import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1433, strictPort: true, watch: { ignored: ['**/src-tauri/**', '**/.qa/**'] } },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: { target: 'es2022' },
});
