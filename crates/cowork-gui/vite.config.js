import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  server: {
    port: 15173,
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // React core
          "vendor-react": ["react", "react-dom"],
          // UI library
          "vendor-antd": ["antd", "@ant-design/icons"],
          // Monaco editor (heavy ~2MB)
          "vendor-monaco": ["@monaco-editor/react", "monaco-editor"],
          // Markdown rendering
          "vendor-markdown": [
            "react-markdown",
            "remark-gfm",
            "rehype-highlight",
            "rehype-raw",
          ],
          // State management
          "vendor-zustand": ["zustand"],
        },
      },
    },
    // Increase chunk size warning limit since we're splitting intentionally
    chunkSizeWarningLimit: 1536,
  },
});
