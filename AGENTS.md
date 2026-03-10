# AGENTS.md

## Project Overview

离线AI聊天软件，使用 Tauri 2.0 + Vue 3 + TypeScript + Ant Design Vue。

核心功能：本地聊天、知识库管理（PDF/Word/TXT/MD/HTML/XLSX）、工具调用（可配置权限）、Skill系统（JSON配置）。

技术栈：Rust (edition 2024) + SQLite + sqlite-vec + Ollama/OpenAI API

---

## Build/Lint/Test Commands

### Build
```bash
cargo tauri dev              # 开发模式
cargo tauri build            # 生产构建
cargo build --release        # 仅Rust后端
npm run build                # 仅前端
```

### Test
```bash
# Rust测试
cargo test                       # 所有测试
cargo test test_name             # 运行单个测试
cargo test --test file_name      # 运行指定测试文件
cargo test --lib module::tests   # 运行指定模块测试
cargo test -- --nocapture        # 显示测试输出

# 前端测试
npm run test
npm run test:unit
npm run test:e2e
```

### Lint & Format
```bash
cargo clippy                 # Rust lint
cargo clippy --fix          # 自动修复
cargo fmt                   # 格式化Rust代码
cargo fmt --check           # 检查格式

npm run lint                # 前端lint
npm run lint:fix            # 自动修复
npm run type-check          # TypeScript检查
```

---

## Code Style Guidelines

### Rust

#### Imports
```rust
// 1. 标准库
use std::collections::HashMap;
use std::path::PathBuf;

// 2. 外部依赖
use serde::{Deserialize, Serialize};
use tauri::command;
use sqlx::SqlitePool;

// 3. 内部模块
use crate::knowledge::Document;
use crate::llm::OllamaClient;
```

#### Formatting
- 使用 `cargo fmt` 自动格式化
- 最大行宽：100字符
- 缩进：4空格

#### Types
```rust
// 使用类型别名简化复杂类型
pub type AppResult<T> = Result<T, AppError>;

// 自定义错误类型
#[derive(Debug)]
pub enum AppError {
    Database(String),
    DocumentParse(String),
    LlmRequest(String),
    ToolPermission(String),
}

// 结构体字段使用camelCase序列化
#[derive(Serialize)]
pub struct Message {
    #[serde(rename = "sessionId")]
    session_id: String,
}
```

#### Naming
- 函数/变量/模块：`snake_case`
- 类型/结构体/枚举：`PascalCase`
- 常量：`SCREAMING_SNAKE_CASE`
- 文件名：`snake_case.rs`

#### Error Handling
```rust
// 使用标准库Result + 自定义错误
pub fn parse_document(path: &Path) -> AppResult<String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::DocumentParse(e.to_string()))?;
    Ok(content)
}

// Tauri命令返回 Result<T, String>
#[command]
pub async fn send_message(message: String) -> Result<String, String> {
    // ...处理逻辑
    Ok(response)
}
```

#### Async
- 使用 `tokio` 运行时
- 异步函数：`async fn` + `.await`
- 文件IO使用 `tokio::fs`

#### Documentation
```rust
/// 函数说明
/// 
/// # Arguments
/// * `param` - 参数说明
/// 
/// # Returns
/// 返回值说明
/// 
/// # Errors
/// 错误情况说明
pub fn function_name(param: Type) -> AppResult<ReturnType> {
```

#### Tauri Commands
```rust
#[command]
pub async fn command_name(
    param: String,
    state: State<'_, AppState>,
) -> Result<ResponseType, String> {
    state.handler
        .process(param)
        .await
        .map_err(|e| e.to_string())
}
```

### Vue/TypeScript

#### File Naming
- 组件：`PascalCase.vue` (例: `ChatMessage.vue`)
- 组合式函数：`useXxx.ts`
- Store：`useXxxStore.ts`
- 类型：`xxx.ts` (例: `message.ts`)

#### Component Structure
```vue
<template>
  <!-- HTML -->
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Message } from '@/types/message'

// Props & Emits
interface Props {
  message: Message
}
const props = defineProps<Props>()

const emit = defineEmits<{
  send: [content: string]
}>()

// State
const inputValue = ref('')

// Computed
const displayText = computed(() => props.message.content)

// Methods
function handleClick() {
  emit('send', inputValue.value)
}
</script>

<style scoped>
/* CSS */
</style>
```

#### Types
```typescript
// types/message.ts
export interface Message {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  metadata?: Record<string, unknown>
}

// 使用 type 而非 interface（除非需要扩展）
export type ChatRole = 'user' | 'assistant' | 'system'
```

#### State Management (Pinia)
```typescript
// stores/useChatStore.ts
import { defineStore } from 'pinia'
import type { Message } from '@/types/message'

export const useChatStore = defineStore('chat', {
  state: () => ({
    messages: [] as Message[],
  }),
  
  getters: {
    lastMessage: (state) => state.messages.at(-1),
  },
  
  actions: {
    async sendMessage(content: string) {
      const response = await invoke<Message>('send_message', { content })
      this.messages.push(response)
    },
  },
})
```

#### Tauri Integration
```typescript
import { invoke } from '@tauri-apps/api/core'

// 调用后端命令
const result = await invoke<ResponseType>('command_name', {
  param1: value1,
  param2: value2,
})
```

---

## Project Structure

```
compliance_assistant/
├── src-tauri/          # Rust后端
│   ├── src/
│   │   ├── main.rs    # 入口
│   │   ├── lib.rs     # 库根
│   │   ├── chat/      # 聊天模块
│   │   ├── knowledge/ # 知识库
│   │   ├── tools/     # 工具调用
│   │   ├── skills/    # Skill系统
│   │   ├── llm/       # 模型适配
│   │   └── storage/   # 数据持久化
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                # Vue前端
│   ├── views/         # 页面
│   ├── components/    # 组件
│   ├── stores/        # Pinia状态
│   ├── composables/   # 组合式函数
│   ├── types/         # TS类型
│   └── utils/         # 工具函数
├── tests/              # 集成测试
└── docs/               # 文档
```

---

## Module Organization (Rust)

每个功能模块独立目录，包含：
- `mod.rs` - 模块入口，导出公共接口
- 子模块文件 - 具体实现
- `tests` 模块 - 单元测试

```rust
// src/chat/mod.rs
mod session;
mod message;

pub use session::Session;
pub use message::Message;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        // ...
    }
}
```

---

## Testing Conventions

### Rust
- 单元测试：`#[cfg(test)] mod tests` 内嵌在模块中
- 测试命名：`test_<function>_<scenario>`
- 集成测试：`tests/` 目录
- 使用 `#[tokio::test]` 测试异步函数

### Frontend
- 单元测试：Vitest
- E2E测试：Playwright
- 测试文件：`*.test.ts` / `*.spec.ts`

---

## Important Notes

- **异步IO**：所有文件操作使用 `tokio::fs`
- **安全**：敏感数据使用系统密钥环存储
- **权限**：工具调用前必须检查权限配置
- **隐私**：向量嵌入使用本地模型，数据不离境
- **错误**：错误信息不包含敏感路径或数据