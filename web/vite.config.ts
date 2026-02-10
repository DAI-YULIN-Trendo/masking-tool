import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  // Using relative base path './' makes it work on both GitHub Pages and Tauri.
  base: './',
  server: {
    port: 5766,
    fs: {
      allow: ['..'],
    },
  },
  build: {
    outDir: 'dist',
  },
})
