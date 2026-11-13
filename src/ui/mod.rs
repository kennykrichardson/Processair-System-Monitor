mod dashboard;

use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    dashboard::render(frame, app);
}
