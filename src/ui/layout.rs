use ratatui::{
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::ui::{panels, theme};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(frame.area());

    let main_area = chunks[0];
    let status_bar = chunks[1];

    // 3-panel layout
    let panels = Layout::horizontal([
        Constraint::Percentage(20), // Platforms
        Constraint::Percentage(30), // Conversations
        Constraint::Percentage(50), // Messages
    ])
    .split(main_area);

    panels::platform_list::render(frame, app, panels[0]);
    panels::conversation_list::render(frame, app, panels[1]);
    panels::message_view::render(frame, app, panels[2]);

    // Status bar
    let help_line = if app.show_help {
        Line::from(vec![
            Span::styled("Tab", theme::ACTIVE_BORDER),
            Span::styled(" switch | ", theme::HELP_TEXT),
            Span::styled("j/k", theme::ACTIVE_BORDER),
            Span::styled(" navigate | ", theme::HELP_TEXT),
            Span::styled("Enter", theme::ACTIVE_BORDER),
            Span::styled(" select | ", theme::HELP_TEXT),
            Span::styled("r", theme::ACTIVE_BORDER),
            Span::styled(" suggest | ", theme::HELP_TEXT),
            Span::styled("e", theme::ACTIVE_BORDER),
            Span::styled(" edit | ", theme::HELP_TEXT),
            Span::styled("?", theme::ACTIVE_BORDER),
            Span::styled(" help | ", theme::HELP_TEXT),
            Span::styled("q", theme::ACTIVE_BORDER),
            Span::styled(" quit", theme::HELP_TEXT),
        ])
    } else {
        Line::from(vec![
            Span::styled("quick-reply", theme::ACTIVE_BORDER),
            Span::styled(" | Press ", theme::HELP_TEXT),
            Span::styled("?", theme::ACTIVE_BORDER),
            Span::styled(" for help", theme::HELP_TEXT),
        ])
    };

    frame.render_widget(Paragraph::new(help_line), status_bar);
}
