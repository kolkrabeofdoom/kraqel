import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { fileURLToPath } from "node:url";

const host = process.env.TAURI_DEV_HOST;
const e2eCommandAdapter = fileURLToPath(new URL("./src/e2e/tauri-core-mock.ts", import.meta.url));

export default defineConfig(({ mode }) => ({
  plugins: [react()],
  clearScreen: false,
  build: {
    rollupOptions: {
      input:
        process.env.KRAQEL_ISOLATION_UI_TEST === "1"
          ? {
              main: fileURLToPath(new URL("./index.html", import.meta.url)),
              isolation: fileURLToPath(new URL("./isolation-test.html", import.meta.url)),
            }
          : fileURLToPath(new URL("./index.html", import.meta.url)),
    },
  },
  resolve: {
    alias:
      process.env.KRAQEL_E2E === "1" || mode === "e2e"
        ? [{ find: "@tauri-apps/api/core", replacement: e2eCommandAdapter }]
        : [],
  },
  server: {
    host: host || false,
    port: 1420,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
