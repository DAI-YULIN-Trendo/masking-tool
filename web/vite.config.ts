import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: '/masking-tool/',
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
