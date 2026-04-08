use crate::config::Config;
use crate::models::{Conversation, Message};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    Main,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsSection {
    Sync,
    Ai,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Platforms,
    Conversations,
    Messages,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Search,
}

#[derive(Debug)]
pub struct AiSuggestion {
    pub text: String,
    pub loading: bool,
}

pub struct App {
    pub running: bool,
    pub active_panel: Panel,
    pub input_mode: InputMode,
    pub platform_names: Vec<String>,
    pub platform_status: Vec<ConnectionStatus>,
    pub selected_platform: usize,
    pub conversations: Vec<Conversation>,
    pub selected_conversation: usize,
    pub messages: Vec<Message>,
    pub selected_message: usize,
    pub suggestion: Option<AiSuggestion>,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub notifications: Vec<String>,
    pub show_help: bool,
    pub config: Config,
    pub view: AppView,
    pub settings_section: SettingsSection,
    pub settings_selected: usize,
    pub sync_status: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
}

impl App {
    pub fn new(config: Config) -> Self {
        let platform_names = vec![
            "Telegram".to_string(),
            "iMessage".to_string(),
            "WhatsApp".to_string(),
            "Slack".to_string(),
            "Messenger".to_string(),
        ];
        let platform_status = vec![ConnectionStatus::Disconnected; platform_names.len()];

        // Placeholder conversations for initial TUI testing
        let conversations = vec![
            Conversation {
                id: "1".to_string(),
                platform: "Telegram".to_string(),
                participants: vec!["John".to_string()],
                last_active: chrono::Utc::now(),
                last_message_preview: Some("Hey, are we meeting today?".to_string()),
                unread_count: 1,
            },
            Conversation {
                id: "2".to_string(),
                platform: "Telegram".to_string(),
                participants: vec!["Sarah".to_string()],
                last_active: chrono::Utc::now(),
                last_message_preview: Some("Check out this link".to_string()),
                unread_count: 0,
            },
        ];

        let messages = vec![
            Message {
                id: "m1".to_string(),
                conversation_id: "1".to_string(),
                platform: "Telegram".to_string(),
                sender: "John".to_string(),
                content: "Hey, are we meeting today?".to_string(),
                timestamp: chrono::Utc::now(),
                is_read: false,
                is_outgoing: false,
            },
            Message {
                id: "m2".to_string(),
                conversation_id: "1".to_string(),
                platform: "Telegram".to_string(),
                sender: "You".to_string(),
                content: "Yes! 3pm at the usual place".to_string(),
                timestamp: chrono::Utc::now(),
                is_read: true,
                is_outgoing: true,
            },
        ];

        Self {
            running: true,
            active_panel: Panel::Platforms,
            input_mode: InputMode::Normal,
            platform_names,
            platform_status,
            selected_platform: 0,
            conversations,
            selected_conversation: 0,
            messages,
            selected_message: 0,
            suggestion: None,
            input_buffer: String::new(),
            input_cursor: 0,
            notifications: Vec::new(),
            show_help: false,
            config,
            view: AppView::Main,
            settings_section: SettingsSection::Sync,
            settings_selected: 0,
            sync_status: "Starting...".to_string(),
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Platforms => Panel::Conversations,
            Panel::Conversations => Panel::Messages,
            Panel::Messages => Panel::Platforms,
        };
    }

    pub fn prev_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Platforms => Panel::Messages,
            Panel::Conversations => Panel::Platforms,
            Panel::Messages => Panel::Conversations,
        };
    }

    pub fn select_next(&mut self) {
        match self.active_panel {
            Panel::Platforms => {
                if !self.platform_names.is_empty() {
                    self.selected_platform =
                        (self.selected_platform + 1) % self.platform_names.len();
                }
            }
            Panel::Conversations => {
                if !self.conversations.is_empty() {
                    self.selected_conversation =
                        (self.selected_conversation + 1) % self.conversations.len();
                }
            }
            Panel::Messages => {
                if !self.messages.is_empty() {
                    self.selected_message = (self.selected_message + 1) % self.messages.len();
                }
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.active_panel {
            Panel::Platforms => {
                if !self.platform_names.is_empty() {
                    self.selected_platform = self
                        .selected_platform
                        .checked_sub(1)
                        .unwrap_or(self.platform_names.len() - 1);
                }
            }
            Panel::Conversations => {
                if !self.conversations.is_empty() {
                    self.selected_conversation = self
                        .selected_conversation
                        .checked_sub(1)
                        .unwrap_or(self.conversations.len() - 1);
                }
            }
            Panel::Messages => {
                if !self.messages.is_empty() {
                    self.selected_message = self
                        .selected_message
                        .checked_sub(1)
                        .unwrap_or(self.messages.len() - 1);
                }
            }
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
