import { defineConfig } from "vite";

// Tauri serves the frontend from `src/` (Vite root) on a fixed port.
// See docs/tech-stack.html for the intended project layout.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  root: "src",
  // Prevent Vite from clearing Rust compiler errors in the terminal.
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_"],
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: {
      // Rust sources are watched by the Tauri CLI, not Vite.
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // Output to repo-root/dist (root is `src/`, so `../dist`).
    outDir: "../dist",
    emptyOutDir: true,
    target: "es2021",
    minify: false,
    sourcemap: true,
  },
});
