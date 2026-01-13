use log::{debug, info};
use ratatui::{backend::Backend, DefaultTerminal, Frame, Terminal};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use tokio::time::{sleep, Duration};

use crate::api_client::ApiClient;

use super::{components, event::handle_event, AppComponent, AppError, AppEvent, AppMode, File};

pub type AppState = Rc<RefCell<State>>;

#[derive(Default, Debug)]
pub struct State {
    initialized: bool,
    dirty: bool,
    quitting: bool,
    pub file: Option<File>,
}

pub struct App {
    api_client: ApiClient,
    state: AppState,
    components: components::Components,
    mode: AppMode,
}

impl App {
    pub fn new(api_client: ApiClient) -> Self {
        let state = Rc::new(RefCell::new(State {
            dirty: true,
            ..State::default()
        }));

        let components = components::Components::new(state.clone());

        App {
            api_client,
            state,
            mode: AppMode::Normal,
            components,
        }
    }

    pub fn startup(&mut self, file: Option<PathBuf>) -> Result<DefaultTerminal, AppError> {
        let mut state = self.state.borrow_mut();
        if state.initialized {
            return Err(AppError::AlreadyInitialized);
        }

        if let Some(path) = file {
            state.file = Some(File::from_path(path)?);
        }

        let terminal = ratatui::init();
        state.initialized = true;
        Ok(terminal)
    }

    pub async fn run<T: Backend>(&mut self, mut terminal: Terminal<T>) -> Result<(), AppError> {
        if !self.state.borrow().initialized {
            return Err(AppError::NotInitialized);
        }

        loop {
            self.handle_event().await?;

            // Needed to borrow state inside
            let (quitting, dirty) = {
                let state = self.state.borrow();
                (state.quitting, state.dirty)
            };

            if quitting {
                info! {"Quitting application..."}
                break;
            }

            if dirty {
                terminal.draw(|frame| self.draw(frame))?;
            }

            self::sleep(Duration::from_millis(16)).await;
        }

        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.state.borrow_mut().initialized = false;
        ratatui::restore();
    }

    fn load_file(&mut self) -> Result<(), AppError> {
        let path = PathBuf::from("./examples/long.yaml");
        self.state.borrow_mut().file = Some(File::from_path(path)?);
        Ok(())
    }

    fn write_file(&self) {
        let mut state = self.state.borrow_mut();
        if let Some(file) = &mut state.file {
            file.write();
        }
    }

    async fn handle_event(&mut self) -> std::io::Result<()> {
        if let Some(event) = handle_event(&self.mode)? {
            self.state.borrow_mut().dirty =
                self.handle_app_events(&event).await || self.handle_component_events(&event);
        }
        Ok(())
    }

    async fn handle_app_events(&mut self, event: &AppEvent) -> bool {
        match event {
            AppEvent::ChangeMode(m) => {
                self.mode = m.clone();
                true
            }
            AppEvent::Load => {
                // TODO: This should load a modal, not the file
                match self.load_file() {
                    Ok(()) => {}
                    Err(_e) => {
                        todo!()
                    }
                }
                true
            }
            AppEvent::Write => {
                self.write_file();
                true
            }
            AppEvent::DumpDebug => {
                info!("App State: {:#?}", self.state.borrow());
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
                self.state.borrow_mut().quitting = true;
                true
            }
            _ => false,
        }
    }

    fn handle_component_events(&mut self, event: &AppEvent) -> bool {
        // TODO: Allow components to push events too
        self.components.handle_event(&self.mode, event)
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.components.draw(&self.mode, frame, frame.area());
        self.state.borrow_mut().dirty = false;
    }
}
