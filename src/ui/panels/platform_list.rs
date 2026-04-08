use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, ConnectionStatus, Panel};
use crate::ui::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Platforms;
    let border_style = if is_active {
        theme::ACTIVE_BORDER
    } else {
        theme::INACTIVE_BORDER
    };

    let items: Vec<ListItem> = app
        .platform_names
        .iter()
        .zip(app.platform_status.iter())
        .enumerate()
        .map(|(i, (name, status))| {
            let status_indicator = match status {
                ConnectionStatus::Connected => Span::styled("[*]", theme::STATUS_CONNECTED),
                ConnectionStatus::Disconnected => Span::styled("[ ]", theme::STATUS_DISCONNECTED),
                ConnectionStatus::Connecting => Span::styled("[~]", theme::STATUS_CONNECTING),
            };

            let name_style = if i == app.selected_platform && is_active {
                theme::SELECTED_ITEM
            } else {
                theme::NORMAL_ITEM
            };

            ListItem::new(Line::from(vec![
                status_indicator,
                Span::raw(" "),
                Span::styled(name.clone(), name_style),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(" Platforms ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items).block(block);

    let mut state = ListState::default();
    if is_active {
        state.select(Some(app.selected_platform));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
