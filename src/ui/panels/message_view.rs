use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Panel};
use crate::ui::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Messages;
    let border_style = if is_active {
        theme::ACTIVE_BORDER
    } else {
        theme::INACTIVE_BORDER
    };

    let has_suggestion = app.suggestion.is_some();

    let chunks = if has_suggestion {
        Layout::vertical([Constraint::Min(3), Constraint::Length(5)]).split(area)
    } else {
        Layout::vertical([Constraint::Min(3)]).split(area)
    };

    // Message list
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let style = if msg.is_outgoing {
                theme::OUTGOING_MSG
            } else {
                theme::INCOMING_MSG
            };

            let time = msg.timestamp.format("%H:%M").to_string();
            let line = Line::from(vec![
                Span::styled(format!("[{time}] "), theme::HELP_TEXT),
                Span::styled(format!("{}: ", msg.sender), style),
                Span::styled(&msg.content, style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let block = Block::default()
        .title(" Messages ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items).block(block);
    frame.render_widget(list, chunks[0]);

    // AI Suggestion panel
    if let Some(suggestion) = &app.suggestion {
        let suggestion_text = if suggestion.loading {
            "Thinking..."
        } else {
            &suggestion.text
        };

        let suggestion_block = Block::default()
            .title(" AI Suggestion ")
            .borders(Borders::ALL)
            .border_style(theme::SUGGESTION_STYLE);

        let suggestion_paragraph = Paragraph::new(Line::from(Span::styled(
            suggestion_text,
            theme::SUGGESTION_STYLE,
        )))
        .block(suggestion_block)
        .wrap(Wrap { trim: true });

        frame.render_widget(suggestion_paragraph, chunks[1]);
    }
}
