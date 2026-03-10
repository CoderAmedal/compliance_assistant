# Compliance Assistant

离线AI聊天助手，支持知识库管理和工具调用。

## 快速开始

### 开发环境要求

- Node.js 18+
- Rust 1.70+
- pnpm 或 npm

### 安装依赖

```bash
# 安装前端依赖
npm install

# Rust依赖会自动安装
```

### 开发模式

```bash
cargo tauri dev
```

### 生产构建

```bash
cargo tauri build
```

## 功能特性

- 本地聊天对话
- 知识库管理（支持 PDF/Word/TXT/Markdown/HTML/XLSX）
- 工具调用框架（可配置权限）
- Skill 技能系统（JSON配置）
- 支持 Ollama 和 OpenAI API

## 项目结构

- `src-tauri/` - Rust 后端
- `src/` - Vue 3 前端
- `docs/skills/` - Skill 配置示例

## 技术栈

- Tauri 2.0
- Vue 3 + TypeScript
- Ant Design Vue
- SQLite + sqlite-vec
- Ollama / OpenAI API

## 开发指南

查看 [AGENTS.md](./AGENTS.md) 了解详细的开发规范和代码风格。

## 许可证

MIT