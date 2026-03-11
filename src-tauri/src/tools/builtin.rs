use super::{Tool, ToolDefinition};
use crate::AppResult;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_read".to_string(),
            description: "读取指定路径的文件内容。支持文本文件、PDF、Word文档等格式。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要读取的文件路径"
                    },
                    "max_lines": {
                        "type": "integer",
                        "description": "最多读取的行数，默认为100"
                    }
                },
                "required": ["path"]
            }),
            require_confirmation: true,
        }
    }

    async fn execute(&self, params: Value) -> AppResult<Value> {
        let path = params["path"].as_str()
            .ok_or_else(|| crate::AppError::ToolPermission("Missing 'path' parameter".to_string()))?;
        
        let max_lines = params["max_lines"].as_u64().unwrap_or(100) as usize;
        
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| crate::AppError::DocumentParse(format!("Failed to read file: {}", e)))?;
        
        let lines: Vec<&str> = content.lines().take(max_lines).collect();
        let result = lines.join("\n");
        
        Ok(serde_json::json!({
            "path": path,
            "content": result,
            "total_lines": content.lines().count(),
            "returned_lines": lines.len()
        }))
    }
}

pub struct KnowledgeSearchTool {
    pool: sqlx::SqlitePool,
    embedding_service: crate::llm::EmbeddingService,
}

impl KnowledgeSearchTool {
    pub fn new(pool: sqlx::SqlitePool, embedding_service: crate::llm::EmbeddingService) -> Self {
        Self { pool, embedding_service }
    }
}

#[async_trait]
impl Tool for KnowledgeSearchTool {
    fn name(&self) -> &str {
        "search_knowledge"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "search_knowledge".to_string(),
            description: "搜索知识库中与查询相关的内容。用于检索已上传文档中的信息。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索查询内容"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "返回结果数量，默认为5"
                    }
                },
                "required": ["query"]
            }),
            require_confirmation: false,
        }
    }

    async fn execute(&self, params: Value) -> AppResult<Value> {
        let query = params["query"].as_str()
            .ok_or_else(|| crate::AppError::ToolPermission("Missing 'query' parameter".to_string()))?;
        
        let limit = params["limit"].as_u64().unwrap_or(5) as usize;
        
        let retriever = crate::knowledge::Retriever::new(self.pool.clone())
            .with_embedding(self.embedding_service.clone());
        
        let chunks = retriever.search(query, limit).await?;
        
        let results: Vec<Value> = chunks
            .into_iter()
            .map(|c| serde_json::json!({
                "document_id": c.document_id,
                "content": c.content,
                "chunk_index": c.chunk_index
            }))
            .collect();
        
        Ok(serde_json::json!({
            "query": query,
            "results": results,
            "total": results.len()
        }))
    }
}

pub struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_search".to_string(),
            description: "在互联网上搜索信息（需要网络连接）。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索关键词"
                    }
                },
                "required": ["query"]
            }),
            require_confirmation: true,
        }
    }

    async fn execute(&self, params: Value) -> AppResult<Value> {
        let query = params["query"].as_str()
            .ok_or_else(|| crate::AppError::ToolPermission("Missing 'query' parameter".to_string()))?;
        
        Ok(serde_json::json!({
            "query": query,
            "results": [],
            "message": "Web search is not implemented yet. This is a placeholder."
        }))
    }
}

pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "calculator".to_string(),
            description: "执行数学计算。支持基本运算和数学函数。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "数学表达式，如 '2 + 3 * 4' 或 'sqrt(16)'"
                    }
                },
                "required": ["expression"]
            }),
            require_confirmation: false,
        }
    }

    async fn execute(&self, params: Value) -> AppResult<Value> {
        let expression = params["expression"].as_str()
            .ok_or_else(|| crate::AppError::ToolPermission("Missing 'expression' parameter".to_string()))?;
        
        let result = self.evaluate_expression(expression)?;
        
        Ok(serde_json::json!({
            "expression": expression,
            "result": result
        }))
    }
}

impl CalculatorTool {
    fn evaluate_expression(&self, expr: &str) -> AppResult<f64> {
        let expr = expr.replace(" ", "");
        
        let mut tokens: Vec<String> = Vec::new();
        let mut current = String::new();
        
        for c in expr.chars() {
            if c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')' {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(c.to_string());
            } else if c.is_numeric() || c == '.' {
                current.push(c);
            }
        }
        if !current.is_empty() {
            tokens.push(current);
        }
        
        let result = self.parse_expression(&tokens, &mut 0)?;
        Ok(result)
    }
    
    fn parse_expression(&self, tokens: &[String], pos: &mut usize) -> AppResult<f64> {
        let mut result = self.parse_term(tokens, pos)?;
        
        while *pos < tokens.len() {
            match tokens[*pos].as_str() {
                "+" => {
                    *pos += 1;
                    result += self.parse_term(tokens, pos)?;
                }
                "-" => {
                    *pos += 1;
                    result -= self.parse_term(tokens, pos)?;
                }
                _ => break,
            }
        }
        
        Ok(result)
    }
    
    fn parse_term(&self, tokens: &[String], pos: &mut usize) -> AppResult<f64> {
        let mut result = self.parse_factor(tokens, pos)?;
        
        while *pos < tokens.len() {
            match tokens[*pos].as_str() {
                "*" => {
                    *pos += 1;
                    result *= self.parse_factor(tokens, pos)?;
                }
                "/" => {
                    *pos += 1;
                    let divisor = self.parse_factor(tokens, pos)?;
                    if divisor == 0.0 {
                        return Err(crate::AppError::ToolPermission("Division by zero".to_string()));
                    }
                    result /= divisor;
                }
                _ => break,
            }
        }
        
        Ok(result)
    }
    
    fn parse_factor(&self, tokens: &[String], pos: &mut usize) -> AppResult<f64> {
        if *pos >= tokens.len() {
            return Err(crate::AppError::ToolPermission("Unexpected end of expression".to_string()));
        }
        
        if tokens[*pos] == "(" {
            *pos += 1;
            let result = self.parse_expression(tokens, pos)?;
            if *pos >= tokens.len() || tokens[*pos] != ")" {
                return Err(crate::AppError::ToolPermission("Missing closing parenthesis".to_string()));
            }
            *pos += 1;
            return Ok(result);
        }
        
        let num: f64 = tokens[*pos].parse()
            .map_err(|_| crate::AppError::ToolPermission(format!("Invalid number: {}", tokens[*pos])))?;
        *pos += 1;
        Ok(num)
    }
}