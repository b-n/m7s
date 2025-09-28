use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use crate::app::{AppComponent, AppMode};

#[derive(Default)]
pub struct Info {}

impl AppComponent for Info {
    fn draw(&self, mode: &AppMode, frame: &mut Frame, area: Rect) {
        let message = match mode {
            AppMode::Normal => vec![
                "(Enter) to enter input mode, ".into(),
                "(q)uit, ".into(),
                "<arrows> to navigate".into(),
            ],
            AppMode::Input => vec!["<ESC> to go back to normal mode.".into()],
            AppMode::Command => vec![":".into()],
        };
        frame.render_widget(Paragraph::new(Text::from(Line::from(message))), area);
    }
}
