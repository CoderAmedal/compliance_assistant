mod loader;
mod runner;

pub use loader::SkillLoader;
pub use runner::SkillRunner;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub prompt: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstance {
    pub skill: Skill,
    pub parameters: serde_json::Value,
}