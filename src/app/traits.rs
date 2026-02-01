use ratatui::{layout::Rect, Frame};

use super::{AppError, AppEvent, AppMode};

pub trait AppComponent {
    fn draw(&mut self, mode: &AppMode, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, _mode: &AppMode, _event: &AppEvent) -> Result<bool, AppError> {
        Ok(false)
    }
}
