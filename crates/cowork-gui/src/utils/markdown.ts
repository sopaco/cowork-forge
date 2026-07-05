// 共享的 markdown 插件配置：避免在多个组件里重复初始化 react-markdown 插件栈，
// 并通过显式注册语言子集，避免引入 rehype-highlight 默认的 35 种 common 语言
// （common 语言包 ~600KB，实际项目只用其中一小部分）。
//
// 默认 rehype-highlight 会引入 lowlight/lib/common（35 种语言）。
// 通过传 languages 选项，tree-shaking 会消除未使用的 common 导入
// （lowlight package.json 声明了 sideEffects: false）。
//
// 新增语言：从 highlight.js/lib/languages/<name> 导入对应 LanguageFn 即可。

import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeRaw from 'rehype-raw';
import type { PluggableList } from 'unified';
import type { LanguageFn } from 'lowlight';

import rust from 'highlight.js/lib/languages/rust';
import typescript from 'highlight.js/lib/languages/typescript';
import javascript from 'highlight.js/lib/languages/javascript';
import json from 'highlight.js/lib/languages/json';
import bash from 'highlight.js/lib/languages/bash';
import shell from 'highlight.js/lib/languages/shell';
import python from 'highlight.js/lib/languages/python';
import yaml from 'highlight.js/lib/languages/yaml';
import ini from 'highlight.js/lib/languages/ini';
import markdown from 'highlight.js/lib/languages/markdown';
import xml from 'highlight.js/lib/languages/xml';
import css from 'highlight.js/lib/languages/css';
import sql from 'highlight.js/lib/languages/sql';
import go from 'highlight.js/lib/languages/go';
import java from 'highlight.js/lib/languages/java';
import plaintext from 'highlight.js/lib/languages/plaintext';
import dockerfile from 'highlight.js/lib/languages/dockerfile';
import makefile from 'highlight.js/lib/languages/makefile';
import diff from 'highlight.js/lib/languages/diff';

// 显式注册的语言子集：覆盖本项目实际会渲染的代码块语言。
// key 是语言名；react-markdown 渲染 ```rust 代码块时会按这个名字查找。
const languages: Record<string, LanguageFn> = {
  rust,
  typescript,
  javascript,
  json,
  bash,
  shell,
  python,
  yaml,
  ini, // 也用于 TOML（highlight.js 没有独立 toml，走 ini）
  markdown,
  xml, // 也用于 HTML / SVG
  css,
  sql,
  go,
  java,
  plaintext,
  dockerfile,
  makefile,
  diff,
};

// remark 插件：GFM 表格 / 删除线 / 任务列表
export const remarkPlugins: PluggableList = [remarkGfm];

// 完整 rehype 插件栈：语法高亮（子集化） + 允许原始 HTML
// tuple 形式 [plugin, options] 是 PluggableList 支持的写法
export const fullRehypePlugins: PluggableList = [
  [rehypeHighlight, { languages }],
  rehypeRaw,
];
