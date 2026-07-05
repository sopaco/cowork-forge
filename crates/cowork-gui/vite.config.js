import { defineConfig } from "vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import babel from "@rolldown/plugin-babel";
import path from "path";

// Vite 8.1 默认使用 Rolldown（Rust 实现）作为打包器，JS/TS 转译默认用 oxc（Rust）
// React Compiler 1.0 GA：通过 plugin-react v6 推荐的 reactCompilerPreset + @rolldown/plugin-babel 接入
// 参考: https://github.com/vitejs/vite-plugin-react#react-compiler

const isProd = process.env.NODE_ENV === "production";

export default defineConfig({
  plugins: [
    react(),
    babel({
      presets: [reactCompilerPreset({ target: "19" })],
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  server: {
    port: 15173,
  },
  // Vite 8.1: esbuild 选项已弃用（被 oxc 替代）。
  // 用 define + 死代码消除来剥除 console/debugger，跨 minifier 都管用。
  // dev 模式下保留 console 便于调试；prod 模式下把 console.* 调用变成 void 0，
  // 然后 Rolldown minifier 会通过 tree-shaking 死代码消除掉。
  define: isProd
    ? {
        // 让 `console.log(...)` 等调用变成 `void 0(...)`，minifier 视为死代码消除
        "console.log": "0",
        "console.info": "0",
        "console.debug": "0",
        "console.trace": "0",
      }
    : {},
  build: {
    // Vite 8: Rolldown 默认开启
    target: "esnext",
    cssCodeSplit: true,
    chunkSizeWarningLimit: 1536,
    // rolldown minifier 默认在 production 启用，会消除被 define 替换为常量的 console 调用
    minify: isProd ? "rolldown" : false,
    rollupOptions: {
      output: {
        // Vite 8 + Rolldown: manualChunks 必须是函数形式，对象形式不再支持
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (id.includes("react-dom") || id.includes("/react/")) return "vendor-react";
            if (id.includes("antd") || id.includes("@ant-design")) return "vendor-antd";
            if (id.includes("@monaco-editor") || id.includes("monaco-editor")) return "vendor-monaco";
            if (
              id.includes("react-markdown") ||
              id.includes("remark-gfm") ||
              id.includes("rehype-highlight") ||
              id.includes("rehype-raw")
            ) {
              return "vendor-markdown";
            }
            if (id.includes("zustand")) return "vendor-zustand";
          }
          return undefined;
        },
      },
    },
  },
});
