use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, Panel};
use crate::ui::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Conversations;
    let border_style = if is_active {
        theme::ACTIVE_BORDER
    } else {
        theme::INACTIVE_BORDER
    };

    let items: Vec<ListItem> = app
        .conversations
        .iter()
        .enumerate()
        .map(|(i, conv)| {
            let name_style = if i == app.selected_conversation && is_active {
                theme::SELECTED_ITEM
            } else {
                theme::NORMAL_ITEM
            };

            let mut spans = vec![Span::styled(conv.display_name().to_string(), name_style)];

            if conv.unread_count > 0 {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!(" {} ", conv.unread_count),
                    theme::UNREAD_BADGE,
                ));
            }

            let mut lines = vec![Line::from(spans)];

            if let Some(preview) = &conv.last_message_preview {
                lines.push(Line::from(Span::styled(
                    format!("  {preview}"),
                    theme::HELP_TEXT,
                )));
            }

            ListItem::new(lines)
        })
        .collect();

    let block = Block::default()
        .title(" Conversations ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items).block(block);

    let mut state = ListState::default();
    if is_active {
        state.select(Some(app.selected_conversation));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
