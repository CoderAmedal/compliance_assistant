use super::Skill;
use crate::tools::ToolExecutor;
use crate::llm::{LlmClient, ChatMessage};
use crate::AppResult;

pub struct SkillRunner<C: LlmClient> {
    llm_client: C,
    tool_executor: ToolExecutor,
}

impl<C: LlmClient> SkillRunner<C> {
    pub fn new(llm_client: C, tool_executor: ToolExecutor) -> Self {
        Self {
            llm_client,
            tool_executor,
        }
    }
    
    pub async fn run(&self, skill: &Skill, params: serde_json::Value) -> AppResult<String> {
        let prompt = self.interpolate_prompt(&skill.prompt, &params);
        
        let mut messages = vec![ChatMessage::system(format!(
            "You are using the skill: {}.\n{}\n\nAvailable tools: {}",
            skill.name,
            skill.description,
            skill.tools.join(", ")
        ))];
        
        messages.push(ChatMessage::user(prompt));
        
        let response = self.llm_client.chat(messages).await?;
        
        Ok(response)
    }
    
    fn interpolate_prompt(&self, template: &str, params: &serde_json::Value) -> String {
        let mut result = template.to_string();
        
        if let serde_json::Value::Object(map) = params {
            for (key, value) in map {
                let placeholder = format!("{{{}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }
        
        result
    }
}