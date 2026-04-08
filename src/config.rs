use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub platforms: PlatformConfigs,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PlatformConfigs {
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,
    #[serde(default)]
    pub imessage: Option<IMessageConfig>,
    #[serde(default)]
    pub whatsapp: Option<WhatsAppConfig>,
    #[serde(default)]
    pub slack: Option<SlackConfig>,
    #[serde(default)]
    pub messenger: Option<MessengerConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelegramConfig {
    pub api_id: i32,
    pub api_hash: String,
    pub phone: String,
    #[serde(default = "default_sync_depth")]
    pub sync_depth: String,
    #[serde(default = "default_sync_recent_count")]
    pub sync_recent_count: usize,
    #[serde(default = "default_build_memory")]
    pub build_memory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IMessageConfig {
    #[serde(default = "default_imessage_db")]
    pub db_path: PathBuf,
    #[serde(default = "default_sync_depth")]
    pub sync_depth: String,
    #[serde(default = "default_sync_recent_count")]
    pub sync_recent_count: usize,
    #[serde(default = "default_build_memory")]
    pub build_memory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WhatsAppConfig {
    #[serde(default = "default_bridge_port")]
    pub bridge_port: u16,
    #[serde(default = "default_sync_depth")]
    pub sync_depth: String,
    #[serde(default = "default_sync_recent_count")]
    pub sync_recent_count: usize,
    #[serde(default = "default_build_memory")]
    pub build_memory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackConfig {
    pub user_token: String,
    #[serde(default = "default_sync_depth")]
    pub sync_depth: String,
    #[serde(default = "default_sync_recent_count")]
    pub sync_recent_count: usize,
    #[serde(default = "default_build_memory")]
    pub build_memory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessengerConfig {
    #[serde(default = "default_messenger_bridge_port")]
    pub bridge_port: u16,
    #[serde(default = "default_sync_depth")]
    pub sync_depth: String,
    #[serde(default = "default_sync_recent_count")]
    pub sync_recent_count: usize,
    #[serde(default = "default_build_memory")]
    pub build_memory: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AiConfig {
    #[serde(default = "default_claude_binary")]
    pub claude_binary: String,
    #[serde(default = "default_suggestion_timeout")]
    pub suggestion_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryConfig {
    #[serde(default = "default_recent_messages")]
    #[allow(dead_code)]
    pub recent_messages_count: usize,
    #[serde(default = "default_search_results")]
    #[allow(dead_code)]
    pub search_results_count: usize,
    #[serde(default = "default_profile_dir")]
    pub profile_dir: PathBuf,
}

impl Config {
    pub fn load() -> crate::error::Result<Self> {
        let config_path = config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> crate::error::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::error::QuickReplyError::Config(format!("Config serialization failed: {e}"))
        })?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            tick_rate_ms: default_tick_rate(),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            claude_binary: default_claude_binary(),
            suggestion_timeout_secs: default_suggestion_timeout(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            recent_messages_count: default_recent_messages(),
            search_results_count: default_search_results(),
            profile_dir: default_profile_dir(),
        }
    }
}

pub(crate) fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("quick-reply")
        .join("config.toml")
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("quick-reply")
}

fn default_tick_rate() -> u64 {
    250
}

fn default_imessage_db() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library/Messages/chat.db")
}

fn default_bridge_port() -> u16 {
    9100
}

fn default_messenger_bridge_port() -> u16 {
    9101
}

fn default_claude_binary() -> String {
    "claude".to_string()
}

fn default_suggestion_timeout() -> u64 {
    30
}

fn default_recent_messages() -> usize {
    20
}

fn default_search_results() -> usize {
    5
}

fn default_profile_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("quick-reply")
        .join("profiles")
}

fn default_sync_depth() -> String {
    "recent".to_string()
}

fn default_sync_recent_count() -> usize {
    50
}

fn default_build_memory() -> bool {
    true
}
