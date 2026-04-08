use ratatui::style::{Color, Modifier, Style};

pub const ACTIVE_BORDER: Style = Style::new().fg(Color::Cyan);
pub const INACTIVE_BORDER: Style = Style::new().fg(Color::DarkGray);
pub const SELECTED_ITEM: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Cyan)
    .add_modifier(Modifier::BOLD);
pub const NORMAL_ITEM: Style = Style::new().fg(Color::White);
pub const OUTGOING_MSG: Style = Style::new().fg(Color::Green);
pub const INCOMING_MSG: Style = Style::new().fg(Color::White);
pub const SUGGESTION_STYLE: Style = Style::new()
    .fg(Color::Yellow)
    .add_modifier(Modifier::ITALIC);
pub const UNREAD_BADGE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Red)
    .add_modifier(Modifier::BOLD);
pub const STATUS_CONNECTED: Style = Style::new().fg(Color::Green);
pub const STATUS_DISCONNECTED: Style = Style::new().fg(Color::Red);
pub const STATUS_CONNECTING: Style = Style::new().fg(Color::Yellow);
pub const HELP_TEXT: Style = Style::new().fg(Color::DarkGray);
