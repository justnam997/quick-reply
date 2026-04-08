mod ai;
mod app;
mod config;
mod error;
mod memory;
mod models;
mod platform;
mod storage;
mod sync;
mod ui;

use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use app::{AiSuggestion, App, InputMode, Panel};
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to file (don't pollute TUI)
    let log_file = std::fs::File::create("/tmp/quick-reply.log").ok();
    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_writer(file)
            .with_ansi(false)
            .init();
    }

    let config = Config::load().unwrap_or_default();

    // Initialize database
    let data_dir = config.general.data_dir.clone();
    std::fs::create_dir_all(&data_dir).ok();
    let db = storage::sqlite::Database::open(&data_dir.join("quick-reply.db")).await?;

    // Initialize AI
    let claude = ai::claude::ClaudeCli::new(&config.ai).await;

    // Initialize conversation archive
    let archive = memory::archive::ConversationArchive::new(db.connection().clone());

    // Build platform manager and register configured adapters
    let mut manager = platform::manager::PlatformManager::new();
    register_platforms(&mut manager, &config);

    // Connect all platforms (non-blocking — failures are logged)
    manager.connect_all().await;

    // Create the app
    let mut app = App::new(config);

    // Update platform statuses
    for (i, name) in app.platform_names.iter().enumerate() {
        app.platform_status[i] = manager.status(name);
    }

    // Load conversations for the first connected platform
    if let Some(platform) = app.platform_names.first() {
        if let Ok(convs) = manager.fetch_conversations(platform).await {
            if !convs.is_empty() {
                // Archive conversations
                for conv in &convs {
                    archive.archive_conversation(conv).await.ok();
                }
                app.conversations = convs;
            }
        }
    }

    // Set up notification channel
    let mut notification_rx = manager.take_notification_receiver();
    manager.start_notification_listeners().await;

    // Channel for async AI suggestion results
    let (suggestion_tx, mut suggestion_rx) = mpsc::channel::<String>(1);

    // Terminal setup
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    // Main event loop
    let tick_rate = Duration::from_millis(app.config.general.tick_rate_ms);

    while app.running {
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Check for AI suggestion results (non-blocking)
        if let Ok(text) = suggestion_rx.try_recv() {
            app.suggestion = Some(AiSuggestion {
                text,
                loading: false,
            });
        }

        // Check for new message notifications (non-blocking)
        if let Some(ref mut rx) = notification_rx {
            while let Ok(notif) = rx.try_recv() {
                app.notifications.push(format!(
                    "[{}] {}: {}",
                    notif.platform, notif.sender, notif.preview
                ));
                // Keep only last 10 notifications
                if app.notifications.len() > 10 {
                    app.notifications.remove(0);
                }
            }
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => {
                        handle_normal_input(
                            &mut app,
                            key.code,
                            key.modifiers,
                            &manager,
                            &archive,
                            &claude,
                            &suggestion_tx,
                        )
                        .await
                    }
                    InputMode::Editing => handle_editing_input(&mut app, key.code, &manager).await,
                    InputMode::Search => handle_search_input(&mut app, key.code),
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn register_platforms(manager: &mut platform::manager::PlatformManager, config: &Config) {
    if let Some(tg) = &config.platforms.telegram {
        manager.register(Box::new(platform::telegram::TelegramAdapter::new(
            tg.clone(),
            &config.general.data_dir,
        )));
    }

    if let Some(im) = &config.platforms.imessage {
        manager.register(Box::new(platform::imessage::IMessageAdapter::new(
            im.clone(),
        )));
    }

    if let Some(wa) = &config.platforms.whatsapp {
        manager.register(Box::new(platform::whatsapp::WhatsAppAdapter::new(
            wa.clone(),
        )));
    }

    if let Some(sl) = &config.platforms.slack {
        manager.register(Box::new(platform::slack::SlackAdapter::new(sl.clone())));
    }

    if let Some(ms) = &config.platforms.messenger {
        manager.register(Box::new(platform::messenger::MessengerAdapter::new(
            ms.clone(),
        )));
    }
}

async fn handle_normal_input(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    manager: &platform::manager::PlatformManager,
    archive: &memory::archive::ConversationArchive,
    claude: &ai::claude::ClaudeCli,
    suggestion_tx: &mpsc::Sender<String>,
) {
    match code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Tab => app.next_panel(),
        KeyCode::BackTab => app.prev_panel(),
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Char('?') => app.show_help = !app.show_help,

        // Enter: load conversations when on Platforms panel, load messages when on Conversations
        KeyCode::Enter => match app.active_panel {
            Panel::Platforms => {
                if let Some(platform) = app.platform_names.get(app.selected_platform) {
                    if let Ok(convs) = manager.fetch_conversations(platform).await {
                        app.conversations = convs;
                        app.selected_conversation = 0;
                        app.next_panel();
                    }
                }
            }
            Panel::Conversations => {
                if let Some(conv) = app.conversations.get(app.selected_conversation) {
                    let platform = &conv.platform;
                    let conv_id = &conv.id;
                    if let Ok(msgs) = manager.fetch_messages(platform, conv_id, 30).await {
                        // Archive fetched messages
                        archive.archive_messages(&msgs).await.ok();
                        app.messages = msgs;
                        app.selected_message = app.messages.len().saturating_sub(1);
                        app.next_panel();
                    }
                }
            }
            Panel::Messages => {
                // Approve and send AI suggestion
                if let Some(suggestion) = &app.suggestion {
                    if !suggestion.loading {
                        if let Some(conv) = app.conversations.get(app.selected_conversation) {
                            match manager
                                .send_message(&conv.platform, &conv.id, &suggestion.text)
                                .await
                            {
                                Ok(confirm) => {
                                    app.notifications.push(format!(
                                        "Sent to {} (msg: {})",
                                        conv.display_name(),
                                        confirm.message_id
                                    ));
                                }
                                Err(e) => {
                                    app.notifications.push(format!("Send failed: {e}"));
                                }
                            }
                            app.suggestion = None;
                        }
                    }
                }
            }
        },

        // 'r' - request AI suggestion
        KeyCode::Char('r') => {
            if claude.is_available() && !app.messages.is_empty() {
                app.suggestion = Some(AiSuggestion {
                    text: String::new(),
                    loading: true,
                });

                let contact_name = app
                    .conversations
                    .get(app.selected_conversation)
                    .map(|c| c.display_name().to_string())
                    .unwrap_or_else(|| "contact".to_string());

                // Load contact profile
                let profile_summary = memory::profile::ContactProfile::load(
                    &app.config.memory.profile_dir,
                    app.conversations
                        .get(app.selected_conversation)
                        .map(|c| c.platform.as_str())
                        .unwrap_or("unknown"),
                    app.conversations
                        .get(app.selected_conversation)
                        .map(|c| c.id.as_str())
                        .unwrap_or("0"),
                )
                .ok()
                .flatten()
                .map(|p| p.summary());

                // Semantic search for relevant past context
                let search = memory::search::SemanticSearch::new(archive);
                let search_terms =
                    memory::search::SemanticSearch::extract_search_terms(&app.messages);
                let relevant_past = if !search_terms.is_empty() {
                    search
                        .search_relevant(&search_terms.join(" "), 5)
                        .await
                        .unwrap_or_default()
                } else {
                    vec![]
                };

                let context = ai::context::ContextAssembler::assemble(
                    &contact_name,
                    profile_summary.as_deref(),
                    &app.messages,
                    &relevant_past,
                );

                // Spawn suggestion in background
                let tx = suggestion_tx.clone();
                let binary = app.config.ai.claude_binary.clone();
                let timeout_secs = app.config.ai.suggestion_timeout_secs;
                let ai_config = crate::config::AiConfig {
                    claude_binary: binary,
                    suggestion_timeout_secs: timeout_secs,
                };

                tokio::spawn(async move {
                    let cli = ai::claude::ClaudeCli::new(&ai_config).await;
                    match cli.suggest_reply(&context, &contact_name).await {
                        Ok(result) => {
                            tx.send(result.text).await.ok();
                        }
                        Err(e) => {
                            tx.send(format!("[Error: {e}]")).await.ok();
                        }
                    }
                });
            }
        }

        // 'e' - edit suggestion before sending
        KeyCode::Char('e') => {
            if let Some(suggestion) = &app.suggestion {
                if !suggestion.loading {
                    app.input_buffer = suggestion.text.clone();
                    app.input_cursor = app.input_buffer.len();
                    app.input_mode = InputMode::Editing;
                }
            }
        }

        KeyCode::Char('/') => {
            app.input_mode = InputMode::Search;
            app.input_buffer.clear();
        }

        KeyCode::Esc => {
            app.suggestion = None;
        }

        _ => {}
    }
}

async fn handle_editing_input(
    app: &mut App,
    code: KeyCode,
    manager: &platform::manager::PlatformManager,
) {
    match code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Enter => {
            if !app.input_buffer.is_empty() {
                // Send the edited message
                if let Some(conv) = app.conversations.get(app.selected_conversation) {
                    match manager
                        .send_message(&conv.platform, &conv.id, &app.input_buffer)
                        .await
                    {
                        Ok(confirm) => {
                            app.notifications.push(format!(
                                "Sent to {} (msg: {})",
                                conv.display_name(),
                                confirm.message_id
                            ));
                        }
                        Err(e) => {
                            app.notifications.push(format!("Send failed: {e}"));
                        }
                    }
                }
                app.suggestion = None;
                app.input_mode = InputMode::Normal;
                app.input_buffer.clear();
            }
        }
        KeyCode::Char(c) => {
            app.input_buffer.insert(app.input_cursor, c);
            app.input_cursor += 1;
        }
        KeyCode::Backspace => {
            if app.input_cursor > 0 {
                app.input_cursor -= 1;
                app.input_buffer.remove(app.input_cursor);
            }
        }
        KeyCode::Left => {
            app.input_cursor = app.input_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            if app.input_cursor < app.input_buffer.len() {
                app.input_cursor += 1;
            }
        }
        _ => {}
    }
}

fn handle_search_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Enter => {
            // TODO: Execute FTS5 search via archive
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        _ => {}
    }
}
