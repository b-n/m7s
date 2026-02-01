use ratatui::{
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    Frame,
};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::app::{AppComponent, AppEvent, AppMode};

#[derive(Default)]
pub struct Airline {
    loaded_path: Option<PathBuf>,
}

impl Airline {
    pub fn new(_: Sender<AppEvent>) -> Self {
        Self { loaded_path: None }
    }
}

impl AppComponent for Airline {
    fn draw(&mut self, mode: &AppMode, frame: &mut Frame, area: Rect) {
        let path = self
            .loaded_path
            .clone()
            .map_or("None".into(), |p| p.display().to_string());

        let airline_message = vec![
            format!(" {} ", mode.display_text()).bold().bg(Color::Green),
            " ".into(),
            format!("File: {path}").fg(Color::Black),
        ];
        frame.render_widget(Line::from(airline_message).bg(Color::Indexed(54)), area);
    }

    fn handle_event(&mut self, _mode: &AppMode, event: &AppEvent) -> bool {
        match event {
            AppEvent::LoadedFile(path) => {
                self.loaded_path = Some(path.clone());
            }
            _ => return false,
        }
        true
    }
}
