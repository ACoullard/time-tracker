import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
// @ts-expect-error process is a nodejs global
const isTauri = !!process.env.TAURI_ENV_PLATFORM;
// @ts-expect-error process is a nodejs global
const mockDir = process.cwd().replace(/\\/g, "/") + "/src/lib/mocks";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [tailwindcss(), sveltekit()],

  // Swap Tauri API packages for browser-safe mocks when running `pnpm dev` without Tauri.
  resolve: {
    alias: {
      ...(isTauri
        ? {}
        : {
            "@tauri-apps/api/core": mockDir + "/tauri-core.ts",
            "@tauri-apps/api/event": mockDir + "/tauri-event.ts",
            "@tauri-apps/api/webviewWindow": mockDir + "/webview-window.ts",
          }),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
