use log::info;
use ratatui::{backend::Backend, DefaultTerminal, Frame, Terminal};
use tokio::time::{sleep, Duration};

use crate::api_client::ApiClient;

use super::components;
use super::event::handle_event;
use super::{AppComponent, AppError, AppEvent, AppMode};

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
    components: components::Components,
}

impl App {
    pub fn new(api_client: ApiClient) -> Self {
        let state = AppState {
            dirty: true,
            ..AppState::default()
        };
        App {
            api_client,
            state,
            components: components::Components::default(),
        }
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
        if let Some(event) = handle_event(&self.state.mode)? {
            self.state.dirty =
                self.handle_app_events(&event) || self.handle_component_events(&event);
        }
        Ok(())
    }

    fn handle_app_events(&mut self, event: &AppEvent) -> bool {
        match event {
            AppEvent::ChangeMode(m) => {
                self.state.mode = m.clone();
                true
            }
            AppEvent::TerminalResize => true,
            AppEvent::Exit => {
                self.state.quitting = true;
                true
            }
            _ => false,
        }
    }

    fn handle_component_events(&mut self, event: &AppEvent) -> bool {
        // TODO: Allow components to push events too
        self.components.handle_event(&self.state.mode, event)
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.components.draw(&self.state.mode, frame, frame.area());
        self.state.dirty = false;
    }
}
