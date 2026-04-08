pub mod layout;
pub mod panels;
pub mod settings;
pub mod theme;

use ratatui::Frame;

use crate::app::{App, AppView};

pub fn render(frame: &mut Frame, app: &App) {
    match app.view {
        AppView::Main => layout::draw(frame, app),
        AppView::Settings => settings::render(frame, app),
    }
}
