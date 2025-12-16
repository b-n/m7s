use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
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
    viewport: (u16, u16),
}

impl Main {
    fn load_file(&mut self) {
        let path = PathBuf::from("./examples/long.yaml");
        self.file = Some(File::from_path(path));
    }

    fn move_cursor(&mut self, dy: &Delta) {
        // Do nothing if the file is not loaded
        if self.file.is_none() {
            return;
        }

        let current_pos = self.cursor_pos.0;

        let mut cursor_y = match (self.cursor_visible(), dy) {
            (true, Delta::Inc(n)) => current_pos.saturating_add(*n),
            (true, Delta::Dec(n)) => current_pos.saturating_sub(*n),
            (false, Delta::Inc(_)) => self.vertical_scroll,
            (false, Delta::Dec(_)) => {
                // Move cursor to bottom of viewport
                self.vertical_scroll
                    .saturating_add(self.viewport.0 as usize)
                    .saturating_sub(1)
            }
            _ => current_pos,
        };
        // Clamp the cursor to the file length
        // Off by 1 due to 0 index
        let line_count = self.file.as_ref().map_or(0, |f| f.line_count);
        if cursor_y >= line_count {
            cursor_y = line_count - 1;
        }

        // Scroll the view if the cursor left the viewport
        log::debug!(
            "Cursor Y: {cursor_y}, Vertical Scroll: {}, Viewport Height: {}",
            self.vertical_scroll,
            self.viewport.0
        );
        if cursor_y < self.vertical_scroll {
            self.scroll_to(None, Some(cursor_y));
        } else if cursor_y
            >= self
                .vertical_scroll
                .saturating_add(self.viewport.0 as usize)
        {
            self.scroll_to(
                None,
                Some(
                    cursor_y
                        .saturating_sub(self.viewport.0 as usize)
                        .saturating_add(1),
                ),
            );
        }

        // Set the value
        self.cursor_pos.0 = cursor_y;
    }

    fn scroll_to(&mut self, x: Option<usize>, y: Option<usize>) {
        match (x, y) {
            (Some(x), Some(y)) => {
                self.horizontal_scroll = x;
                self.vertical_scroll = y;
            }
            (Some(x), None) => {
                self.horizontal_scroll = x;
            }
            (None, Some(y)) => {
                self.vertical_scroll = y;
            }
            _ => {}
        }

        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    fn scroll(&mut self, dx: isize, dy: isize) {
        let mut vertical_scroll = self.vertical_scroll.saturating_add_signed(dy);
        let mut horizontal_scroll = self.horizontal_scroll.saturating_add_signed(dx);

        let (file_width, file_length) = self
            .file
            .as_ref()
            .map_or((0, 0), |f| (f.max_width, f.line_count));

        if horizontal_scroll >= file_width {
            horizontal_scroll = file_width - 1;
        }

        if vertical_scroll >= file_length {
            vertical_scroll = file_length - 1;
        }

        self.scroll_to(Some(horizontal_scroll), Some(vertical_scroll));
    }

    fn cursor_visible(&self) -> bool {
        let (y, _) = self.cursor_pos;
        y >= self.vertical_scroll
            && y < self
                .vertical_scroll
                .saturating_add(self.viewport.0 as usize)
    }
}

impl Main {
    #[allow(clippy::cast_possible_truncation)]
    fn draw_content(&mut self, _mode: &AppMode, frame: &mut Frame, area: Rect) {
        if let Some(file) = &self.file {
            let (content, max_line) = file.display_lines(self.cursor_pos);

            self.vertical_scroll_state = self.vertical_scroll_state.content_length(content.len());
            self.horizontal_scroll_state = self.horizontal_scroll_state.content_length(max_line);

            let block = Block::new().borders(Borders::RIGHT | Borders::BOTTOM);

            let paragraph = Paragraph::new(Text::from(content))
                .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16))
                .block(block);
            frame.render_widget(paragraph, area);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                area,
                &mut self.vertical_scroll_state,
            );
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                    .begin_symbol(Some("←"))
                    .end_symbol(Some("→")),
                area,
                &mut self.horizontal_scroll_state,
            );
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw_line_numbers(&self, _mode: &AppMode, frame: &mut Frame, area: Rect) {
        let block = Block::new()
            .bg(Color::Indexed(22))
            .padding(Padding::right(1));

        let text = if self.file.is_some() {
            let top = self.vertical_scroll as u16;
            let lines: Vec<Line<'_>> = (0..self.viewport.0)
                .map(|i| {
                    let line_no = i + top;
                    let mut line = Line::from(format!("{line_no}").to_string());
                    if line_no as usize == self.cursor_pos.0 {
                        line = line.bg(Color::Indexed(236));
                    }
                    line
                })
                .collect();
            Text::from(lines)
        } else {
            Text::from("0")
        };

        let paragraph = Paragraph::new(text).right_aligned().block(block);
        frame.render_widget(paragraph, area);
    }
}

impl AppComponent for Main {
    #[allow(clippy::cast_possible_truncation)]
    fn draw(&mut self, mode: &AppMode, frame: &mut Frame, area: Rect) {
        let [line_numbers, main_content] =
            Layout::horizontal([Constraint::Length(6), Constraint::Min(1)]).areas(area);

        // Height and width reduces by 1 for scrollbars
        self.viewport = (
            main_content.height.saturating_sub(1),
            main_content.width.saturating_sub(1),
        );

        frame.render_widget(Block::new().bg(Color::Indexed(22)), line_numbers);
        self.draw_content(mode, frame, main_content);
        self.draw_line_numbers(mode, frame, line_numbers);
    }

    fn handle_event(&mut self, _mode: &AppMode, event: &AppEvent) -> bool {
        match event {
            AppEvent::Load => {
                // TODO: This should load a modal, not the file
                self.load_file();
            }
            AppEvent::CursorY(d) => self.move_cursor(d),
            AppEvent::ScrollX(d) => {
                self.scroll(d.into(), 0);
            }
            AppEvent::ScrollY(d) => {
                self.scroll(0, d.into());
            }
            AppEvent::CursorX(d) => {
                self.cursor_pos.1 = match d {
                    Delta::Inc(_) => 1,
                    _ => 0,
                };
            }
            _ => return false,
        }
        true
    }
}
