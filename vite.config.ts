import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const tauriDevHost = process.env.TAURI_DEV_HOST;
const port = Number(process.env.TAURI_DEV_PORT ?? 5175);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port,
    strictPort: true,
    // Always listen on all interfaces so the dev server is reachable from LAN clients.
    // If you need HMR to connect from a different host/device, set TAURI_DEV_HOST.
    host: true,
    hmr: tauriDevHost
      ? {
          protocol: "ws",
          host: tauriDevHost,
          port,
        }
      : undefined,
    proxy: {
      // Proxy backend API so the frontend can use same-origin requests in web mode.
      // This avoids CORS and avoids requiring port 8080 to be reachable from the LAN.
      "/api": {
        target: "http://127.0.0.1:8080",
        changeOrigin: true,
      },
      "/ws": {
        target: "ws://127.0.0.1:8080",
        ws: true,
      },
    },
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
});