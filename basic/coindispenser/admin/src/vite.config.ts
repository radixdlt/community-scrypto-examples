import { defineConfig } from 'vite'

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: '3001',
    host: '0.0.0.0',
    strictPort: 'true',
  },
})
