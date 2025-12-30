use ratatui::{
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    Frame,
};

use crate::app::{AppComponent, AppMode, AppState};

#[derive(Default)]
pub struct Airline {}

impl Airline {
    pub fn new(_state: AppState) -> Self {
        Self {}
    }
}

impl AppComponent for Airline {
    fn draw(&mut self, mode: &AppMode, frame: &mut Frame, area: Rect) {
        let airline_message = vec![
            format!(" {} ", mode.display_text()).bold().bg(Color::Green),
            " ".into(),
            "File: example.yaml".fg(Color::Black),
        ];
        frame.render_widget(Line::from(airline_message).bg(Color::Indexed(54)), area);
    }
}
