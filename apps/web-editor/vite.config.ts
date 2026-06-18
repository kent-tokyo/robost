import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      // All /api/* and /events, /screenshot → Rust agent
      '/api': 'http://localhost:9921',
      '/events': 'http://localhost:9921',
      '/screenshot': 'http://localhost:9921',
    },
  },
  build: {
    outDir: 'dist',
  },
})
