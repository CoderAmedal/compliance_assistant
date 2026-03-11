<template>
  <div class="markdown-body" ref="containerRef"></div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
import MarkdownIt from 'markdown-it'
import mermaid from 'mermaid'
import hljs from 'highlight.js'

interface Props {
  content: string
}

const props = defineProps<Props>()
const containerRef = ref<HTMLElement | null>(null)

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

function render() {
  if (!containerRef.value) return
  
  containerRef.value.innerHTML = md.render(props.content)
  nextTick(() => {
    renderMermaid()
  })
}

watch(() => props.content, () => {
  render()
}, { immediate: false })

onMounted(() => {
  render()
})
</script>

<style>
@import 'highlight.js/styles/github.css';

.markdown-body {
  font-size: 14px;
  line-height: 1.6;
  color: #333;
}

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

.markdown-body h1 {
  font-size: 2em;
  border-bottom: 1px solid #eaecef;
  padding-bottom: .3em;
}

.markdown-body h2 {
  font-size: 1.5em;
  border-bottom: 1px solid #eaecef;
  padding-bottom: .3em;
}

.markdown-body h3 {
  font-size: 1.25em;
}

.markdown-body h4 {
  font-size: 1em;
}

.markdown-body p {
  margin-top: 0;
  margin-bottom: 16px;
}

.markdown-body ul,
.markdown-body ol {
  margin-top: 0;
  margin-bottom: 16px;
  padding-left: 2em;
}

.markdown-body ul ul,
.markdown-body ul ol,
.markdown-body ol ol,
.markdown-body ol ul {
  margin-top: 0;
  margin-bottom: 0;
}

.markdown-body li {
  margin-top: 0.25em;
}

.markdown-body li + li {
  margin-top: 0.25em;
}

.markdown-body blockquote {
  margin: 0 0 16px;
  padding: 0 1em;
  color: #6a737d;
  border-left: 0.25em solid #dfe2e5;
}

.markdown-body code {
  padding: 0.2em 0.4em;
  margin: 0;
  font-size: 85%;
  background-color: rgba(27, 31, 35, 0.05);
  border-radius: 3px;
}

.markdown-body pre {
  padding: 16px;
  overflow: auto;
  font-size: 85%;
  line-height: 1.45;
  background-color: #f6f8fa;
  border-radius: 6px;
  margin: 0 0 16px;
}

.markdown-body pre code {
  background-color: transparent;
  padding: 0;
  border-radius: 0;
  font-size: 100%;
}

.markdown-body .hljs {
  background: #f6f8fa;
}

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

.markdown-body table th {
  font-weight: 600;
  background-color: #f6f8fa;
}

.markdown-body table tr {
  background-color: #fff;
  border-top: 1px solid #c6cbd1;
}

.markdown-body table tr:nth-child(2n) {
  background-color: #f6f8fa;
}

.markdown-body img {
  max-width: 100%;
  box-sizing: content-box;
  background-color: #fff;
}

.markdown-body a {
  color: #0366d6;
  text-decoration: none;
}

.markdown-body a:hover {
  text-decoration: underline;
}

.markdown-body hr {
  height: 0.25em;
  padding: 0;
  margin: 24px 0;
  background-color: #e1e4e8;
  border: 0;
}

.markdown-body .mermaid {
  background-color: #fff;
  padding: 16px;
  border-radius: 6px;
  margin: 0 0 16px;
  overflow-x: auto;
}

.markdown-body .mermaid svg {
  max-width: 100%;
}
</style>