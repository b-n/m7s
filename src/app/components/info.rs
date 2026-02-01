use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};
use std::string::ToString;
use std::sync::mpsc::Sender;

use crate::app::{AppComponent, AppEvent, AppMode};

#[derive(Default)]
pub struct Info {
    log: Vec<String>,
}

impl Info {
    pub fn new(_: Sender<AppEvent>) -> Self {
        Self { log: Vec::new() }
    }
}

impl AppComponent for Info {
    fn draw(&mut self, mode: &AppMode, frame: &mut Frame, area: Rect) {
        let mut lines = vec![Line::from(match mode {
            AppMode::Normal => vec![
                "(enter)".bold().cyan(),
                " to enter input mode, ".into(),
                "(q)".bold().cyan(),
                "uit, ".into(),
                "<arrows>".bold().cyan(),
                " to navigate".into(),
            ],
            AppMode::Input => vec!["<ESC> to go back to normal mode.".into()],
            AppMode::Command => vec![":".into()],
        })];

        for log_line in &self.log {
            lines.push(Line::from(log_line.clone()));
        }

        let p = Paragraph::new(Text::from(lines.clone())).wrap(Wrap { trim: false });

        frame.render_widget(p, area);
    }

    fn handle_event(&mut self, _mode: &AppMode, event: &AppEvent) -> bool {
        match event {
            AppEvent::Debug(msg) => {
                self.log = msg.split('\n').map(ToString::to_string).collect();
            }
            _ => return false,
        }
        true
    }
}
