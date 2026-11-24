import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ command, mode }) => {
  // Mock Tauri layer disabled - use real Tauri backend (npm run tauri dev).
  // To re-enable mocks for browser-only testing: set useMocks = command === "serve" && !host
  const useMocks = false;

  return {
    plugins: [svelte(), tailwindcss()],
    publicDir: "static",
    resolve: {
      // Vitest transforms modules through Vite's SSR pipeline. Component tests
      // still need Svelte's browser runtime so @testing-library/svelte can mount
      // components into jsdom rather than resolving svelte/index-server.
      conditions: mode === "test" ? ["browser"] : undefined,
      alias: {
        $lib: path.resolve("./src/lib"),
        $press: path.resolve("./src/styles/catalyst"),
        ...(useMocks
          ? {
              "@tauri-apps/api/core": path.resolve("./src/dev/mock-tauri.ts"),
              "@tauri-apps/api/event": path.resolve("./src/dev/mock-event.ts"),
              "@tauri-apps/plugin-dialog": path.resolve("./src/dev/mock-dialog.ts"),
              "@tauri-apps/plugin-opener": path.resolve("./src/dev/mock-opener.ts"),
            }
          : {}),
      },
    },
    clearScreen: false,
    test: {
      // Catalyst frontend tests exercise browser APIs such as DOMParser,
      // localStorage, window, and component mounting. Keep Vitest in jsdom by
      // default so `npm test` matches the environment these modules require.
      environment: "jsdom",
      setupFiles: ["./src/test/setup.ts"],
    },
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
        // QA writes screenshots, DOM dumps and reports while driving the app.
        // Watching those artifacts reloads the webview mid-scenario.
        ignored: [
          "**/src-tauri/**",
          // Project folders are authoritative runtime data. Saving manuscript or
          // metadata files must not make Vite reload the application shell.
          "**/docs/projects/**",
          "**/qa/visual/results/**",
          "**/qa/visual/data/**",
          "**/qa/demo/data/**",
        ],
      },
    },
  };
});
