use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

mod airline;
mod info;
mod main;

pub use airline::Airline;
pub use info::Info;
pub use main::Main;

use super::{AppComponent, AppEvent, AppMode, AppState};

#[derive(Default)]
pub struct Components<'a> {
    main: Main<'a>,
    airline: Airline,
    info: Info,
}

impl Components<'_> {
    pub fn new(state: &AppState) -> Self {
        Self {
            main: Main::new(state.clone()),
            airline: Airline::new(state.clone()),
            info: Info::new(state.clone()),
        }
    }
}

impl AppComponent for Components<'_> {
    fn draw(&mut self, mode: &AppMode, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(4),
        ]);

        let [body_area, airline_area, info_area] = layout.areas(area);

        self.main.draw(mode, frame, body_area);
        self.airline.draw(mode, frame, airline_area);
        self.info.draw(mode, frame, info_area);
    }

    fn handle_event(&mut self, mode: &AppMode, event: &AppEvent) -> bool {
        self.main.handle_event(mode, event)
            || self.airline.handle_event(mode, event)
            || self.info.handle_event(mode, event)
    }
}
