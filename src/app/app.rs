use log::{debug, info};
use ratatui::{backend::Backend, DefaultTerminal, Frame, Terminal};
use std::path::PathBuf;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, RwLock,
};
use tokio::time::{sleep, Duration};

use crate::api_client::ApiClient;

use super::{components, event::poll_input_for_event, AppComponent, AppError, AppEvent, AppMode};

#[derive(Default, PartialEq, Clone)]
enum AppState {
    #[default]
    Uninitialized,
    Initialized,
    Running,
    Quitting,
}

pub struct App<'a> {
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
    api_client: ApiClient,
    components: components::Components<'a>,
    mode: Arc<RwLock<AppMode>>,
    state: Arc<RwLock<AppState>>,
    dirty: bool,
}

impl App<'_> {
    pub fn new(api_client: ApiClient) -> Self {
        let (sender, receiver) = channel();

        let components = components::Components::new(sender.clone());

        App {
            sender: sender.clone(),
            receiver,
            api_client,
            components,
            mode: Arc::new(RwLock::new(AppMode::Normal)),
            state: Arc::new(RwLock::new(AppState::default())),
            dirty: true,
        }
    }

    fn state(&self) -> AppState {
        self.state.read().unwrap().clone()
    }

    fn set_state(&mut self, new_state: AppState) {
        let mut state = self.state.write().unwrap();
        *state = new_state;
    }

    fn mode(&self) -> AppMode {
        self.mode.read().unwrap().clone()
    }

    fn set_mode(&mut self, new_mode: AppMode) {
        let mut mode = self.mode.write().unwrap();
        *mode = new_mode;
    }

    pub fn startup(&mut self) -> Result<DefaultTerminal, AppError> {
        if self.state() != AppState::Uninitialized {
            return Err(AppError::AlreadyInitialized);
        }

        let terminal = ratatui::init();
        self.set_state(AppState::Initialized);
        Ok(terminal)
    }

    fn poll_input(&self) {
        let sender = self.sender.clone();
        let state = self.state.clone();
        let mode = self.mode.clone();

        tokio::spawn(async move {
            loop {
                let mode = mode.read().unwrap().clone();
                if *state.read().unwrap() == AppState::Quitting {
                    break;
                }
                match poll_input_for_event(&mode) {
                    Ok(Some(event)) => {
                        sender.send(event).unwrap();
                    }
                    Ok(None) => {}
                    Err(e) => {
                        log::error!("Error polling input: {e}");
                    }
                }
            }
        });
    }

    pub async fn run<T: Backend>(&mut self, mut terminal: Terminal<T>) -> Result<(), AppError> {
        if self.state() != AppState::Initialized {
            return Err(AppError::NotInitialized);
        }

        self.set_state(AppState::Running);

        // Setup input polling for generating input AppEvent's
        self.poll_input();

        loop {
            // Handle any incoming events
            if let Ok(event) = self.receiver.recv_timeout(Duration::from_millis(10)) {
                self.handle_event(&event).await?;
            }

            if self.state() == AppState::Quitting {
                info!("Quitting application...");
                break;
            }

            if self.dirty {
                terminal.draw(|frame| self.draw(frame))?;
            }

            self::sleep(Duration::from_millis(16)).await;
        }

        Ok(())
    }

    pub fn shutdown(self) {
        if self.state() != AppState::Quitting {
            log::warn!("Shutting down app that is not in quitting state");
        }
        ratatui::restore();
    }

    // Helper function when loading the application to immediately load a file
    pub fn load_file(&mut self, path: PathBuf) -> Result<(), AppError> {
        Ok(self.sender.send(AppEvent::LoadPath(path))?)
    }

    async fn handle_event(&mut self, event: &AppEvent) -> Result<(), AppError> {
        let app_requires_rerender = self.handle_app_events(event).await;
        let component_requires_rerender = self.handle_component_events(event)?;
        self.dirty = app_requires_rerender || component_requires_rerender;
        Ok(())
    }

    async fn handle_app_events(&mut self, event: &AppEvent) -> bool {
        match event {
            AppEvent::ChangeMode(m) => {
                self.set_mode(m.clone());
                true
            }
            AppEvent::TerminalResize => true,
            AppEvent::LoadSpec => {
                // Load a core object spec
                let group = "v1".into();
                let spec = self.api_client.get_group_spec(&group).await.unwrap();
                let path = crate::api_client::QueryPath::new("containers").with_parent("spec");
                let opts = spec.get_kind_path("Pod", &path);
                debug!("Spec: {opts:#?}");

                // Load a group object spec
                let group = ("apps", "v1").into();
                let spec = self.api_client.get_group_spec(&group).await.unwrap();
                let path = crate::api_client::QueryPath::new("containers")
                    .with_parent("spec")
                    .with_parent("template")
                    .with_parent("spec");
                let opts = spec.get_kind_path("Deployment", &path);
                debug!("Spec: {opts:#?}");

                true
            }
            AppEvent::Exit => {
                self.set_state(AppState::Quitting);
                true
            }
            _ => false,
        }
    }

    fn handle_component_events(&mut self, event: &AppEvent) -> Result<bool, AppError> {
        // TODO: Allow components to push events too
        self.components.handle_event(&self.mode(), event)
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.components.draw(&self.mode(), frame, frame.area());
        self.dirty = false;
    }
}
