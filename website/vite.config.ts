import { defineConfig } from 'vite';

export default defineConfig({
  // GitHub Pages base path - set to repo name for project pages
  // Use "/" for custom domains or local development
  base: process.env.VITE_BASE_PATH || '/pixelsrc/',
  server: {
    port: 5173,
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
    assetsDir: 'assets',
  },
  optimizeDeps: {
    exclude: ['@stiwi/pixelsrc-wasm'],
  },
});
