import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

const isDev = process.env.TAURI_DEBUG === "true";

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 5173,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: isDev ? false : "esbuild",
    sourcemap: isDev,
  },
});
