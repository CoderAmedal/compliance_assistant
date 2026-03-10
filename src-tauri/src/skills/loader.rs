use super::Skill;
use crate::{AppError, AppResult};
use std::path::Path;

pub struct SkillLoader;

impl SkillLoader {
    pub fn load_from_file(path: &Path) -> AppResult<Skill> {
        let content = std::fs::read_to_string(path)?;
        let skill: Skill = serde_json::from_str(&content)?;

        if skill.name.is_empty() {
            return Err(AppError::Config("Skill name is required".to_string()));
        }

        Ok(skill)
    }

    pub fn load_from_dir(dir: &Path) -> AppResult<Vec<Skill>> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                match Self::load_from_file(&path) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => eprintln!("Failed to load skill from {:?}: {}", path, e),
                }
            }
        }

        Ok(skills)
    }
}
