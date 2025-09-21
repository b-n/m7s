use log::info;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame, Terminal,
};
use tokio::time::{sleep, Duration};

use crate::api_client::ApiClient;

use super::event::handle_event;
use super::{AppError, AppEvent, AppMode};

#[derive(Default)]
struct AppState {
    mode: AppMode,
    initialized: bool,
    dirty: bool,
    quitting: bool,
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

            self::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub fn shutdown(&self) {
        ratatui::restore();
    }

    fn handle_event(&mut self) -> std::io::Result<()> {
        match handle_event(&self.state.mode)? {
            Some(AppEvent::ChangeMode(m)) => {
                self.state.mode = m;
                self.state.dirty = true;
            }
            Some(AppEvent::Exit) => {
                self.state.quitting = true;
            }
            None => {}
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let main_layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(4),
        ]);

        let [body_area, airline_area, info_area] = main_layout.areas(frame.area());

        let message = match self.state.mode {
            AppMode::Normal => vec![
                "(Enter) to enter input mode, ".into(),
                "(q)uit, ".into(),
                "<arrows> to navigate".into(),
            ],
            AppMode::Input => vec!["<ESC> to go back to normal mode.".into()],
        };
        frame.render_widget(Paragraph::new(Text::from(Line::from(message))), info_area);

        let [line_numbers, main_content] =
            Layout::horizontal([Constraint::Length(6), Constraint::Min(1)]).areas(body_area);

        let airline_message = vec![
            format!(" {} ", self.state.mode.display_text())
                .bold()
                .bg(Color::Green),
            " ".into(),
            "File: example.yaml".fg(Color::Black),
        ];
        frame.render_widget(
            Line::from(airline_message).bg(Color::Indexed(54)),
            airline_area,
        );
        frame.render_widget(Block::new().bg(Color::DarkGray), line_numbers);
    }
}
