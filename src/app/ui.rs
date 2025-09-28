use log::info;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame, Terminal,
};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use crate::api_client::ApiClient;

use super::event::handle_event;
use super::{AppError, AppEvent, AppMode, File};

#[derive(Default)]
struct AppState {
    mode: AppMode,
    initialized: bool,
    dirty: bool,
    quitting: bool,
    cursor_pos: usize,
    file: Option<File>,
}

pub struct App {
    api_client: ApiClient,
    state: AppState,
}

impl App {
    pub fn new(api_client: ApiClient) -> Self {
        let state = AppState {
            dirty: true,
            ..AppState::default()
        };
        App { api_client, state }
    }

    pub fn startup(&mut self) -> Result<DefaultTerminal, AppError> {
        if self.state.initialized {
            return Err(AppError::AlreadyInitialized);
        }
        let terminal = ratatui::init();
        self.state.initialized = true;
        Ok(terminal)
    }

    pub async fn run<T: Backend>(&mut self, mut terminal: Terminal<T>) -> Result<(), AppError> {
        if !self.state.initialized {
            return Err(AppError::NotInitialized);
        }

        loop {
            self.handle_event()?;

            if self.state.quitting {
                info! {"Quitting application..."}
                break;
            }

            if self.state.dirty {
                terminal.draw(|frame| self.draw(frame))?;
            }

            self::sleep(Duration::from_millis(16)).await;
        }

        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.state.initialized = false;
        ratatui::restore();
    }

    fn handle_event(&mut self) -> std::io::Result<()> {
        match handle_event(&self.state.mode)? {
            Some(AppEvent::ChangeMode(m)) => {
                self.state.mode = m;
            }
            Some(AppEvent::Exit) => {
                self.state.quitting = true;
            }
            Some(AppEvent::Load) => {
                // TODO: This should load a modal, not the file
                self.load_file();
            }
            Some(AppEvent::CursorY(dy)) => {
                self.state.cursor_pos = self.state.cursor_pos.saturating_add_signed(dy);
            }
            Some(_) => {
                todo!()
            }
            None => {
                return Ok(());
            }
        }
        self.state.dirty = true;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(4),
        ]);

        let [body_area, airline_area, info_area] = layout.areas(frame.area());

        self.render_main(frame, body_area);
        self.render_airline(frame, airline_area);
        self.render_info(frame, info_area);
    }

    fn render_main(&mut self, frame: &mut Frame, area: Rect) {
        let [line_numbers, main_content] =
            Layout::horizontal([Constraint::Length(6), Constraint::Min(1)]).areas(area);

        frame.render_widget(Block::new().bg(Color::DarkGray), line_numbers);

        if let Some(file) = &self.state.file {
            let content = file.display_lines(self.state.cursor_pos);
            frame.render_widget(Paragraph::new(Text::from(content)), main_content);
        }
    }

    fn render_airline(&mut self, frame: &mut Frame, area: Rect) {
        let airline_message = vec![
            format!(" {} ", self.state.mode.display_text())
                .bold()
                .bg(Color::Green),
            " ".into(),
            "File: example.yaml".fg(Color::Black),
        ];
        frame.render_widget(Line::from(airline_message).bg(Color::Indexed(54)), area);
    }

    fn render_info(&mut self, frame: &mut Frame, area: Rect) {
        let message = match self.state.mode {
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

    fn load_file(&mut self) {
        let path = PathBuf::from("./examples/long.yaml");
        self.state.file = Some(File::from_path(path));
        self.state.dirty = true;
    }
}
