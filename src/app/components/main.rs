use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use tui_textarea::TextArea;

use crate::app::file::Direction;
use crate::app::{AppComponent, AppEvent, AppMode, AppState, Delta};

#[derive(Default)]
struct CursorState {
    byte_offset: u32,
    line: usize,
}

#[derive(Default)]
pub struct Main<'a> {
    state: AppState,
    cursor: CursorState,
    vertical_scroll_state: ScrollbarState,
    horizontal_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    horizontal_scroll: usize,
    viewport: (u16, u16),
    textarea: TextArea<'a>,
}

impl Main<'_> {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            cursor: CursorState::default(),
            vertical_scroll_state: ScrollbarState::default(),
            horizontal_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            horizontal_scroll: 0,
            viewport: (0, 0),
            textarea: TextArea::default(),
        }
    }

    fn set_cursor(&mut self, byte_offset: u32) {
        self.cursor.byte_offset = byte_offset;
        self.cursor.line = self
            .state
            .borrow()
            .file
            .as_ref()
            .expect("File is loaded")
            .line_at_cursor(self.cursor.byte_offset);
    }

    fn cursor_visible(&self) -> bool {
        self.cursor.line >= self.vertical_scroll
            && self.cursor.line
                < self
                    .vertical_scroll
                    .saturating_add(self.viewport.0 as usize)
    }
}

// Movement/scroll helpers
impl Main {
    fn move_cursor_x(&mut self, dx: &Delta) {
        // Do nothing if the file is not loaded
        if self.state.borrow().file.is_none() {
            return;
        }

        let dir = match dx {
            Delta::Inc(n) => Direction::Right(*n),
            Delta::Dec(n) => Direction::Left(*n),
            Delta::Zero => Direction::Right(0),
        };

        self.cursor.byte_offset = self
            .state
            .borrow()
            .file
            .as_ref()
            .expect("File is loaded")
            .navigate_dir(self.cursor.byte_offset, &dir);
    }

    fn move_cursor_y(&mut self, dy: &Delta) {
        // Do nothing if the file is not loaded
        if self.state.borrow().file.is_none() {
            return;
        }

        // Get the new cursor position
        // If the cursor is visible, then use the file navigation
        // If not, choose a line based on the scroll direction, and find the first token
        self.set_cursor(if self.cursor_visible() {
            let dir = match dy {
                Delta::Inc(n) => Direction::Down(*n),
                Delta::Dec(n) => Direction::Up(*n),
                Delta::Zero => Direction::Down(0),
            };
            self.state
                .borrow()
                .file
                .as_ref()
                .expect("File is loaded")
                .navigate_dir(self.cursor.byte_offset, &dir)
        } else {
            log::debug!("Cursor not visible, moving to line");
            let line = match dy {
                Delta::Inc(_) => self.vertical_scroll,
                Delta::Dec(_) => self
                    .vertical_scroll
                    .saturating_add(self.viewport.0 as usize)
                    .saturating_sub(1),
                Delta::Zero => self.cursor.line,
            };
            self.state
                .borrow()
                .file
                .as_ref()
                .expect("File is loaded")
                .first_selectable_at_line(line)
        });

        // Scroll the view if the cursor left the viewport
        if self.cursor.line < self.vertical_scroll {
            self.scroll_to(Some(self.cursor.line), None);
        } else if self.cursor.line
            >= self
                .vertical_scroll
                .saturating_add(self.viewport.0 as usize)
        {
            self.scroll_to(
                Some(
                    self.cursor
                        .line
                        .saturating_sub(self.viewport.0 as usize)
                        .saturating_add(1),
                ),
                None,
            );
        }
    }

    fn scroll_to(&mut self, y: Option<usize>, x: Option<usize>) {
        match (y, x) {
            (Some(y), Some(x)) => {
                self.vertical_scroll = y;
                self.horizontal_scroll = x;
            }
            (Some(y), None) => {
                self.vertical_scroll = y;
            }
            (None, Some(x)) => {
                self.horizontal_scroll = x;
            }
            _ => {}
        }

        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    fn scroll(&mut self, dy: isize, dx: isize) {
        let mut vertical_scroll = self.vertical_scroll.saturating_add_signed(dy);
        let mut horizontal_scroll = self.horizontal_scroll.saturating_add_signed(dx);

        let (file_width, file_length) = self
            .state
            .borrow()
            .file
            .as_ref()
            .map_or((1, 1), |f| (f.max_width, f.line_count));

        if horizontal_scroll >= file_width {
            horizontal_scroll = file_width - 1;
        }

        if vertical_scroll >= file_length {
            vertical_scroll = file_length - 1;
        }

        self.scroll_to(Some(vertical_scroll), Some(horizontal_scroll));
    }
}

// Drawing helpers
impl Main<'_> {
    #[allow(clippy::cast_possible_truncation)]
    fn draw_content(&mut self, _mode: &AppMode, frame: &mut Frame, area: Rect) {
        if let Some(file) = &self.state.borrow().file {
            let (content, max_line) = file.render(self.cursor.byte_offset.try_into().unwrap());

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
    fn draw_line_numbers(&self, _mode: &AppMode, frame: &mut Frame, area: Rect, line_count: usize) {
        let block = Block::new()
            .bg(Color::Indexed(22))
            .padding(Padding::right(1));

        let top = self.vertical_scroll;
        let lines: Vec<Line<'_>> = (top..line_count)
            .map(|i| {
                let line_no = i.saturating_add(1);
                let mut line = Line::from(format!("{line_no}").to_string());
                if i == self.cursor.line {
                    line = line.bg(Color::Indexed(236));
                }
                line
            })
            .collect();

        let paragraph = Paragraph::new(Text::from(lines))
            .right_aligned()
            .block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_editor(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::new().borders(Borders::ALL).title("Editor");

        let editor_area = Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };
        self.textarea.set_block(block);
        frame.render_widget(&self.textarea, editor_area);
    }
}

impl AppComponent for Main<'_> {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn draw(&mut self, mode: &AppMode, frame: &mut Frame, area: Rect) {
        let (line_count, max_width) = self
            .state
            .borrow()
            .file
            .as_ref()
            .map_or((1, 1), |f| (f.line_count, f.max_width));

        self.vertical_scroll_state = self.vertical_scroll_state.content_length(line_count);
        self.horizontal_scroll_state = self.horizontal_scroll_state.content_length(max_width);

        // Calculate the width needed for line numbers based on max viewport size and current
        // scroll. log10 to get number of digits, truncated to remove decimal, and +3 (one for
        // digit, and 2 for padding)
        let line_number_width = f64::from(self.vertical_scroll as u16 + area.height)
            .log10()
            .trunc() as u16
            + 3;

        let [line_numbers, main_content] =
            Layout::horizontal([Constraint::Length(line_number_width), Constraint::Min(1)])
                .areas(area);

        // Height and width reduces by 1 for scrollbars
        self.viewport = (
            main_content.height.saturating_sub(1),
            main_content.width.saturating_sub(1),
        );

        frame.render_widget(Block::new().bg(Color::Indexed(22)), line_numbers);
        self.draw_content(mode, frame, main_content);
        self.draw_line_numbers(mode, frame, line_numbers, line_count);
        if mode == &AppMode::Input {
            self.draw_editor(frame, area);
        }
    }

    fn handle_event(&mut self, mode: &AppMode, event: &AppEvent) -> bool {
        match event {
            AppEvent::CursorY(d) => self.move_cursor_y(d),
            AppEvent::ScrollX(d) => {
                self.scroll(0, d.into());
            }
            AppEvent::ScrollY(d) => {
                self.scroll(d.into(), 0);
            }
            AppEvent::CursorX(d) => {
                self.move_cursor_x(d);
            }
            AppEvent::Info => self
                .state
                .borrow()
                .file
                .as_ref()
                .expect("")
                .info(self.cursor.byte_offset),
            AppEvent::ChangeMode(_mode) => {
                let tokeninfo = self
                    .state
                    .borrow()
                    .file
                    .as_ref()
                    .expect("")
                    .token_info_at_cursor(self.cursor.byte_offset);
                log::info!("Token at cursor: {tokeninfo:?}");
            }
            AppEvent::Raw(e) if mode == &AppMode::Input => {
                self.textarea.input(*e);
            }
            _ => return false,
        }
        true
    }
}
