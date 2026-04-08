use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{QuickReplyError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactProfile {
    pub name: String,
    pub platform: String,
    pub relationship: Option<String>,
    pub facts: Vec<String>,
    pub preferences: Vec<String>,
    pub tone: Option<String>,
    pub notes: Option<String>,
}

impl ContactProfile {
    #[allow(dead_code)]
    pub fn new(name: &str, platform: &str) -> Self {
        Self {
            name: name.to_string(),
            platform: platform.to_string(),
            relationship: None,
            facts: Vec::new(),
            preferences: Vec::new(),
            tone: None,
            notes: None,
        }
    }

    pub fn load(profile_dir: &Path, platform: &str, contact_id: &str) -> Result<Option<Self>> {
        let path = profile_path(profile_dir, platform, contact_id);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let profile: Self = toml::from_str(&content)?;
        Ok(Some(profile))
    }

    #[allow(dead_code)]
    pub fn save(&self, profile_dir: &Path, contact_id: &str) -> Result<()> {
        std::fs::create_dir_all(profile_dir)?;
        let path = profile_path(profile_dir, &self.platform, contact_id);
        let content = toml::to_string_pretty(self)
            .map_err(|e| QuickReplyError::Config(format!("Profile serialization failed: {e}")))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("Name: {}", self.name));
        if let Some(rel) = &self.relationship {
            parts.push(format!("Relationship: {rel}"));
        }
        if let Some(tone) = &self.tone {
            parts.push(format!("Tone: {tone}"));
        }
        if !self.facts.is_empty() {
            parts.push(format!("Facts: {}", self.facts.join("; ")));
        }
        if !self.preferences.is_empty() {
            parts.push(format!("Preferences: {}", self.preferences.join("; ")));
        }
        if let Some(notes) = &self.notes {
            parts.push(format!("Notes: {notes}"));
        }
        parts.join("\n")
    }
}

fn profile_path(profile_dir: &Path, platform: &str, contact_id: &str) -> PathBuf {
    profile_dir.join(format!("{}_{}.toml", platform.to_lowercase(), contact_id))
}
