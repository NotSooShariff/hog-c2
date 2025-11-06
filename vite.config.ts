import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],

  // Production optimizations
  build: {
    // Minify for production
    minify: 'esbuild',

    // Code splitting for better optimization
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'charts': ['recharts'],
        },
      },
    },

    // Source maps - disable in production for security
    sourcemap: false,

    // Asset handling
    assetsInlineLimit: 4096,
    cssCodeSplit: true,

    // Chunk size warnings
    chunkSizeWarningLimit: 1000,
  },

  // Clear the screen on restart
  clearScreen: true,
})
