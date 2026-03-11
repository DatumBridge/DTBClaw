import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  base: "/_app/",
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    outDir: "dist",
  },
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: "http://localhost:5555",
        changeOrigin: true,
      },
      "/ws": {
        target: "ws://localhost:5555",
        ws: true,
      },
      "/pair": {
        target: "http://localhost:5555",
        changeOrigin: true,
      },
      "/health": {
        target: "http://localhost:5555",
        changeOrigin: true,
      },
      "/v1": {
        target: "http://localhost:5555",
        changeOrigin: true,
      },
      "/metrics": {
        target: "http://localhost:5555",
        changeOrigin: true,
      },
    },
  },
});
