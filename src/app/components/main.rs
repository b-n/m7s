use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, Paragraph},
    Frame,
};
use std::path::PathBuf;

use crate::app::{AppComponent, AppEvent, AppMode, File};

#[derive(Default)]
pub struct Main {
    file: Option<File>,
    cursor_pos: usize,
}

impl Main {
    fn load_file(&mut self) {
        let path = PathBuf::from("./examples/long.yaml");
        self.file = Some(File::from_path(path));
    }
}

impl AppComponent for Main {
    fn draw(&self, _mode: &AppMode, frame: &mut Frame, area: Rect) {
        let [line_numbers, main_content] =
            Layout::horizontal([Constraint::Length(6), Constraint::Min(1)]).areas(area);

        frame.render_widget(Block::new().bg(Color::DarkGray), line_numbers);

        if let Some(file) = &self.file {
            let content = file.display_lines(self.cursor_pos);
            frame.render_widget(Paragraph::new(Text::from(content)), main_content);
        }
    }

    fn handle_event(&mut self, _mode: &AppMode, event: &AppEvent) -> bool {
        match event {
            AppEvent::Load => {
                // TODO: This should load a modal, not the file
                self.load_file();
                true
            }
            AppEvent::CursorY(dy) => {
                self.cursor_pos = self.cursor_pos.saturating_add_signed(*dy);
                true
            }
            _ => false,
        }
    }
}
