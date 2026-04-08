pub mod layout;
pub mod panels;
pub mod theme;

use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    layout::draw(frame, app);
}
