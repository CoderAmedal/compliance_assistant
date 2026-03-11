<template>
  <div class="markdown-body" ref="containerRef"></div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
import MarkdownIt from 'markdown-it'
import mermaid from 'mermaid'
import hljs from 'highlight.js'

/**
 * Markdown 渲染组件
 * 
 * 功能特性：
 * 1. 支持标准 Markdown 语法（标题、列表、引用、表格等）
 * 2. 支持 Mermaid 图表渲染（流程图、时序图、甘特图等）
 * 3. 支持代码语法高亮（使用 highlight.js）
 * 4. 自动响应内容变化重新渲染
 * 
 * 使用示例：
 * <MarkdownRenderer :content="markdownText" />
 */
interface Props {
  content: string
}

const props = defineProps<Props>()

const containerRef = ref<HTMLElement | null>(null)

/**
 * 初始化 MarkdownIt 解析器
 * 
 * 配置选项：
 * - html: 允许渲染 HTML 标签
 * - linkify: 自动将 URL 转换为链接
 * - typographer: 启用智能标点转换
 * - breaks: 将换行符转换为 <br> 标签
 * 
 * highlight 函数：自定义代码块渲染逻辑
 * 1. 如果是 mermaid 代码块，包装为 <div class="mermaid"> 供后续渲染
 * 2. 如果指定了语言且 highlight.js 支持，进行语法高亮
 * 3. 否则进行简单的 HTML 转义
 */
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  breaks: true,
  highlight: function (str: string, lang: string) {
    if (lang === 'mermaid') {
      return `<div class="mermaid">${str}</div>`
    }
    
    if (lang && hljs.getLanguage(lang)) {
      try {
        return `<pre class="hljs"><code>${hljs.highlight(str, { language: lang, ignoreIllegals: true }).value}</code></pre>`
      } catch (__) {}
    }
    
    return `<pre class="hljs"><code>${md.utils.escapeHtml(str)}</code></pre>`
  }
})

/**
 * 初始化 Mermaid 配置
 * 
 * 配置说明：
 * - startOnLoad: false - 禁用自动渲染，改用手动控制
 * - theme: 'default' - 使用默认主题（可选：default, dark, forest, neutral）
 * - securityLevel: 'loose' - 允许更宽松的安全策略，支持更复杂的图表
 * - flowchart: 流程图配置
 *   - useMaxWidth: 自适应容器宽度
 *   - htmlLabels: 使用 HTML 标签（支持更丰富的样式）
 * - sequence: 时序图配置
 *   - useMaxWidth: 自适应容器宽度
 *   - wrap: 自动换行
 */
mermaid.initialize({
  startOnLoad: false,
  theme: 'default',
  securityLevel: 'loose',
  flowchart: {
    useMaxWidth: true,
    htmlLabels: true
  },
  sequence: {
    useMaxWidth: true,
    wrap: true
  }
})

/**
 * 渲染所有 Mermaid 图表
 * 
 * 工作流程：
 * 1. 查找容器中所有 .mermaid 元素
 * 2. 遍历每个元素，使用 mermaid.render() 渲染为 SVG
 * 3. 为每个图表生成唯一 ID（避免重复）
 * 4. 错误处理：渲染失败时显示友好的错误提示
 */
async function renderMermaid() {
  if (!containerRef.value) return
  
  const mermaidDivs = containerRef.value.querySelectorAll('.mermaid')
  for (let i = 0; i < mermaidDivs.length; i++) {
    const div = mermaidDivs[i] as HTMLElement
    try {
      const id = `mermaid-${Date.now()}-${i}`
      const { svg } = await mermaid.render(id, div.textContent || '')
      div.innerHTML = svg
    } catch (error) {
      console.error('Mermaid render error:', error)
      div.innerHTML = `<pre style="color: #ff4d4f;">Mermaid 渲染失败</pre>`
    }
  }
}

/**
 * 主渲染函数
 * 
 * 工作流程：
 * 1. 使用 MarkdownIt 将 Markdown 文本转换为 HTML
 * 2. 在 nextTick 后执行 Mermaid 图表渲染
 * 
 * 为什么需要 nextTick：
 * - 确保 DOM 更新完成后再渲染 Mermaid
 * - Mermaid 需要操作真实的 DOM 元素
 */
function render() {
  if (!containerRef.value) return
  
  containerRef.value.innerHTML = md.render(props.content)
  nextTick(() => {
    renderMermaid()
  })
}

/**
 * 监听 props.content 变化
 * 
 * 当外部传入的 Markdown 内容发生变化时，自动重新渲染
 * immediate: false - 避免重复渲染（onMounted 会处理首次渲染）
 */
watch(() => props.content, () => {
  render()
}, { immediate: false })

/**
 * 组件挂载时执行首次渲染
 */
onMounted(() => {
  render()
})
</script>

<style>
/* 引入 highlight.js 的 GitHub 风格主题 */
@import 'highlight.js/styles/github.css';

/* Markdown 容器基础样式 */
.markdown-body {
  font-size: 14px;
  line-height: 1.6;
  color: #333;
}

/* 标题样式 - 使用 GitHub 风格的排版 */
.markdown-body h1,
.markdown-body h2,
.markdown-body h3,
.markdown-body h4,
.markdown-body h5,
.markdown-body h6 {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
}

/* 一级标题 - 添加下边框增强层次感 */
.markdown-body h1 {
  font-size: 2em;
  border-bottom: 1px solid #eaecef;
  padding-bottom: .3em;
}

/* 二级标题 - 添加下边框 */
.markdown-body h2 {
  font-size: 1.5em;
  border-bottom: 1px solid #eaecef;
  padding-bottom: .3em;
}

/* 三级和四级标题 */
.markdown-body h3 {
  font-size: 1.25em;
}

.markdown-body h4 {
  font-size: 1em;
}

/* 段落样式 */
.markdown-body p {
  margin-top: 0;
  margin-bottom: 16px;
}

/* 有序和无序列表 */
.markdown-body ul,
.markdown-body ol {
  margin-top: 0;
  margin-bottom: 16px;
  padding-left: 2em;
}

/* 嵌套列表 - 重置边距 */
.markdown-body ul ul,
.markdown-body ul ol,
.markdown-body ol ol,
.markdown-body ol ul {
  margin-top: 0;
  margin-bottom: 0;
}

/* 列表项间距 */
.markdown-body li {
  margin-top: 0.25em;
}

.markdown-body li + li {
  margin-top: 0.25em;
}

/* 引用块 - 左侧边框 + 浅色文字 */
.markdown-body blockquote {
  margin: 0 0 16px;
  padding: 0 1em;
  color: #6a737d;
  border-left: 0.25em solid #dfe2e5;
}

/* 行内代码 - 浅灰背景 + 圆角 */
.markdown-body code {
  padding: 0.2em 0.4em;
  margin: 0;
  font-size: 85%;
  background-color: rgba(27, 31, 35, 0.05);
  border-radius: 3px;
}

/* 代码块 - 深灰背景 + 滚动条 */
.markdown-body pre {
  padding: 16px;
  overflow: auto;
  font-size: 85%;
  line-height: 1.45;
  background-color: #f6f8fa;
  border-radius: 6px;
  margin: 0 0 16px;
}

/* 代码块内的 code 元素 - 重置样式 */
.markdown-body pre code {
  background-color: transparent;
  padding: 0;
  border-radius: 0;
  font-size: 100%;
}

/* highlight.js 容器背景色 */
.markdown-body .hljs {
  background: #f6f8fa;
}

/* 表格样式 - 边框合并 + 条纹背景 */
.markdown-body table {
  border-spacing: 0;
  border-collapse: collapse;
  margin: 0 0 16px;
  width: 100%;
  overflow: auto;
}

.markdown-body table th,
.markdown-body table td {
  padding: 6px 13px;
  border: 1px solid #dfe2e5;
}

/* 表头 - 加粗 + 浅灰背景 */
.markdown-body table th {
  font-weight: 600;
  background-color: #f6f8fa;
}

/* 表格行 - 白色背景 + 顶部边框 */
.markdown-body table tr {
  background-color: #fff;
  border-top: 1px solid #c6cbd1;
}

/* 斑马纹表格 - 偶数行浅灰背景 */
.markdown-body table tr:nth-child(2n) {
  background-color: #f6f8fa;
}

/* 图片 - 自适应宽度 */
.markdown-body img {
  max-width: 100%;
  box-sizing: content-box;
  background-color: #fff;
}

/* 链接样式 */
.markdown-body a {
  color: #0366d6;
  text-decoration: none;
}

.markdown-body a:hover {
  text-decoration: underline;
}

/* 分隔线样式 */
.markdown-body hr {
  height: 0.25em;
  padding: 0;
  margin: 24px 0;
  background-color: #e1e4e8;
  border: 0;
}

/* Mermaid 图表容器 - 白色背景 + 圆角 + 横向滚动 */
.markdown-body .mermaid {
  background-color: #fff;
  padding: 16px;
  border-radius: 6px;
  margin: 0 0 16px;
  overflow-x: auto;
}

/* Mermaid SVG 图表 - 自适应容器宽度 */
.markdown-body .mermaid svg {
  max-width: 100%;
}
</style>