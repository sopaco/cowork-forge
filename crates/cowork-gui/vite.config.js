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
    // P3: 桌面应用从本地文件系统加载，chunk 大小警告阈值放宽。
    // antd v6 的 CSS-in-JS runtime + 主题/locale 是不可压缩的固定成本，
    // gzip 后 ~470KB 对 Tauri 本地加载无性能影响（<100ms 解析）。
    chunkSizeWarningLimit: 2500,
    // rolldown minifier 默认在 production 启用，会消除被 define 替换为常量的 console 调用
    minify: isProd ? "rolldown" : false,
    rollupOptions: {
      output: {
        // Vite 8 + Rolldown: manualChunks 必须是函数形式，对象形式不再支持
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (id.includes("react-dom") || id.includes("/react/")) return "vendor-react";
            // P2: 不再把 antd 强制合并为单个 chunk。
            // 之前所有 antd 都被打入 vendor-antd，导致 lazy 面板（KnowledgePanel /
            // MemoryPanel / SettingsPanel 等）的 antd 依赖也被提前加载。
            // 现在让 Rolldown 自动 code-split：初始渲染用到的 antd 进入共享 chunk，
            // 仅 lazy 面板用到的组件随面板按需加载。
            // @ant-design/icons 仍合并：图标是 SVG 路径字符串，跨面板高频共用
            if (id.includes("@ant-design/icons")) return "vendor-icons";
            if (id.includes("@monaco-editor") || id.includes("monaco-editor")) return "vendor-monaco";
            if (
              id.includes("react-markdown") ||
              id.includes("remark-gfm") ||
              id.includes("rehype-highlight") ||
              id.includes("rehype-raw") ||
              // P0 后 highlight.js 语言子集和 lowlight 与 react-markdown 一起使用，
              // 合并到同一 chunk 避免拆出微小语言文件
              id.includes("highlight.js") ||
              id.includes("lowlight")
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
