use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::path::PathBuf;

use crate::app::{AppComponent, AppEvent, AppMode, Delta, File};

#[derive(Default)]
pub struct Main {
    file: Option<File>,
    cursor_pos: (usize, usize),
    vertical_scroll_state: ScrollbarState,
    horizontal_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    horizontal_scroll: usize,
}

impl Main {
    fn load_file(&mut self) {
        let path = PathBuf::from("./examples/long.yaml");
        self.file = Some(File::from_path(path));
    }

    fn move_cursor(&mut self, dy: isize) {
        let (mut y, first_element) = self.cursor_pos;
        y = y.saturating_add_signed(dy);

        let line_count = self.file.as_ref().map_or(0, |f| f.line_count);

        if y > line_count {
            y = line_count - 1;
        }

        self.cursor_pos = (y, first_element);
    }

    fn scroll(&mut self, dx: isize, dy: isize) {
        self.vertical_scroll = self.vertical_scroll.saturating_add_signed(dy);
        self.horizontal_scroll = self.horizontal_scroll.saturating_add_signed(dx);

        let (file_width, file_length) = self
            .file
            .as_ref()
            .map_or((0, 0), |f| (f.max_width, f.line_count));

        if self.horizontal_scroll >= file_width {
            self.horizontal_scroll = file_width - 1;
        }

        if self.vertical_scroll >= file_length {
            self.vertical_scroll = file_length - 1;
        }

        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }
}

impl AppComponent for Main {
    #[allow(clippy::cast_possible_truncation)]
    fn draw(&mut self, _mode: &AppMode, frame: &mut Frame, area: Rect) {
        let [line_numbers, main_content] =
            Layout::horizontal([Constraint::Length(6), Constraint::Min(1)]).areas(area);

        frame.render_widget(Block::new().bg(Color::DarkGray), line_numbers);

        if let Some(file) = &self.file {
            let (content, max_line) = file.display_lines(self.cursor_pos);

            self.vertical_scroll_state = self.vertical_scroll_state.content_length(content.len());
            self.horizontal_scroll_state = self.horizontal_scroll_state.content_length(max_line);

            let block = Block::new().borders(Borders::RIGHT | Borders::BOTTOM);

            let paragraph = Paragraph::new(Text::from(content))
                .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16))
                .block(block);
            frame.render_widget(paragraph, main_content);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                main_content,
                &mut self.vertical_scroll_state,
            );
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                    .begin_symbol(Some("←"))
                    .end_symbol(Some("→")),
                main_content,
                &mut self.horizontal_scroll_state,
            );
        }
    }

    fn handle_event(&mut self, _mode: &AppMode, event: &AppEvent) -> bool {
        match event {
            AppEvent::Load => {
                // TODO: This should load a modal, not the file
                self.load_file();
            }
            AppEvent::CursorY(dy) => {
                self.move_cursor(*dy);
            }
            AppEvent::ScrollX(dx) => {
                self.scroll(*dx, 0);
            }
            AppEvent::ScrollY(dy) => {
                self.scroll(0, *dy);
            }
            AppEvent::CursorX(d) => {
                self.cursor_pos.1 = match d {
                    Delta::Inc(_) => 1,
                    Delta::Dec(_) => 0,
                };
            }
            _ => return false,
        }
        true
    }
}
