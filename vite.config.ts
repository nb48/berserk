import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    host: true
  },
  optimizeDeps: {
    exclude: ['@babylonjs/core', '@babylonjs/loaders', '@babylonjs/materials']
  },
  build: {
    rollupOptions: {
      external: []
    }
  }
});
