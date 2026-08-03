import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  base: "./",
  plugins: [vue(), wasm(), topLevelAwait()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    // 开发代理：前端请求 /api 相对路径时转发到本地 axum（23001）
    // 生产（docker 同源部署）不需要代理，/api 直接命中 axum
    proxy: {
      "/api": {
        target: "http://localhost:23001",
        changeOrigin: true,
      },
    },
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
    fs: {
      strict: false,
    },
  },


  build: {
    target: "esnext",
    rollupOptions: {

    },
  },
}));

