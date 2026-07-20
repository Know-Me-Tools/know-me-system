import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [react(), tailwindcss()],
  // This package is a workspace-local Tauri guest binding. Excluding it from
  // Vite's dependency pre-bundle keeps newly generated Rust command wrappers
  // available immediately instead of serving a stale cached export surface.
  optimizeDeps: { exclude: ['@prometheus-ags/tauri-plugin-gen-ui'] },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
    proxy: {
      '/api': {
        target: process.env.VITE_UAR_PROXY_TARGET || 'http://127.0.0.1:1906',
        changeOrigin: true,
      },
    },
  },
  resolve: { alias: { '@': path.resolve(__dirname, './src') } },
  envPrefix: ['VITE_', 'TAURI_'],
  // safari15, NOT Tauri's scaffold default of safari13: the app's dependencies emit
  // destructuring that esbuild cannot downlevel that far — `vite build` failed with 103
  // "Transforming destructuring to the configured target environment is not supported yet"
  // errors. Nothing caught it because `tsc` and `vite dev` both pass; only the PRODUCTION
  // bundle (the one Tauri actually ships) goes through esbuild's transform.
  //
  // safari14 still fails; 15 is the lowest that builds, verified by bisecting. Safari 15
  // ships with macOS 12, which is above Tauri 2's own macOS floor — so this narrows the
  // supported range slightly, deliberately, rather than shipping a bundle that cannot be
  // built at all.
  build: { target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari15', minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false, sourcemap: !!process.env.TAURI_ENV_DEBUG },
})
