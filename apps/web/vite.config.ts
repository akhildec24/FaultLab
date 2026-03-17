import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'node:path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@faultlab/simulation-client': resolve(__dirname, '../../packages/simulation-client/src/index.ts'),
      '@faultlab/simulation-client/worker': resolve(__dirname, '../../packages/simulation-client/src/worker.ts'),
    },
  },
  server: {
    port: 5173,
  },
  worker: {
    format: 'es',
  },
})
