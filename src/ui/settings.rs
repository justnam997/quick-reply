use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, SettingsSection};
use crate::ui::theme;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Tab bar
        Constraint::Min(3),    // Content
        Constraint::Length(1), // Help line
    ])
    .split(frame.area());

    render_tabs(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_help(frame, chunks[2]);
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Sync", "AI", "General"];
    let selected = match app.settings_section {
        SettingsSection::Sync => 0,
        SettingsSection::Ai => 1,
        SettingsSection::General => 2,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Settings "))
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.settings_section {
        SettingsSection::Sync => render_sync_section(frame, app, area),
        SettingsSection::Ai => render_ai_section(frame, app, area),
        SettingsSection::General => render_general_section(frame, app, area),
    }
}

fn render_sync_section(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // Section title
        Constraint::Length(7), // Platform sync settings
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Memory title
        Constraint::Length(7), // Memory status
        Constraint::Min(0),    // Fill
    ])
    .split(area);

    // Platform sync settings header
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Platform Sync Settings",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))),
        chunks[0],
    );

    // Platform list with sync depth
    let platforms = &app.platform_names;
    let items: Vec<ListItem> = platforms
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let depth = get_platform_sync_depth(&app.config, name);
            let is_selected = app.settings_selected == i;

            let style = if is_selected {
                theme::SELECTED_ITEM
            } else {
                theme::NORMAL_ITEM
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("  {:<12}", name), style),
                Span::styled(format!("[{:<8}]", depth), Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::styled(
                    app.sync_status.clone(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, chunks[1]);

    // Memory status header
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Memory Status",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))),
        chunks[3],
    );

    // Memory status per platform
    let memory_items: Vec<ListItem> = platforms
        .iter()
        .map(|name| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {:<12}", name), theme::NORMAL_ITEM),
                Span::styled(
                    "0/0 embedded | 0 profiles",
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let memory_list = List::new(memory_items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(memory_list, chunks[4]);
}

fn render_ai_section(frame: &mut Frame, app: &App, area: Rect) {
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  Claude binary:  ", theme::NORMAL_ITEM),
            Span::styled(
                &app.config.ai.claude_binary,
                Style::default().fg(Color::Yellow),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  Timeout:        ", theme::NORMAL_ITEM),
            Span::styled(
                format!("{}s", app.config.ai.suggestion_timeout_secs),
                Style::default().fg(Color::Yellow),
            ),
        ])),
    ];

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::NONE)
            .title("  AI Settings"),
    );
    frame.render_widget(list, area);
}

fn render_general_section(frame: &mut Frame, app: &App, area: Rect) {
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  Data directory:  ", theme::NORMAL_ITEM),
            Span::styled(
                app.config.general.data_dir.display().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  Tick rate:       ", theme::NORMAL_ITEM),
            Span::styled(
                format!("{}ms", app.config.general.tick_rate_ms),
                Style::default().fg(Color::Yellow),
            ),
        ])),
    ];

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::NONE)
            .title("  General Settings"),
    );
    frame.render_widget(list, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" sections | ", theme::HELP_TEXT),
        Span::styled("j/k", Style::default().fg(Color::Cyan)),
        Span::styled(" navigate | ", theme::HELP_TEXT),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" edit | ", theme::HELP_TEXT),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" back", theme::HELP_TEXT),
    ]);
    frame.render_widget(Paragraph::new(help), area);
}

fn get_platform_sync_depth(config: &crate::config::Config, platform: &str) -> String {
    match platform {
        "Telegram" => config
            .platforms
            .telegram
            .as_ref()
            .map(|c| c.sync_depth.clone()),
        "iMessage" => config
            .platforms
            .imessage
            .as_ref()
            .map(|c| c.sync_depth.clone()),
        "WhatsApp" => config
            .platforms
            .whatsapp
            .as_ref()
            .map(|c| c.sync_depth.clone()),
        "Slack" => config
            .platforms
            .slack
            .as_ref()
            .map(|c| c.sync_depth.clone()),
        "Messenger" => config
            .platforms
            .messenger
            .as_ref()
            .map(|c| c.sync_depth.clone()),
        _ => None,
    }
    .unwrap_or_else(|| "recent".to_string())
}
