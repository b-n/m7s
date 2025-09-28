use log::debug;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use tokio::time::Duration;

use super::AppMode;

#[derive(Debug)]
pub enum AppEvent {
    ChangeMode(AppMode),
    Exit,
    Submit,
    Load,
    CursorY(isize),
    ScrollX(isize),
    ScrollY(isize),
}

pub fn handle_event(mode: &AppMode) -> io::Result<Option<AppEvent>> {
    match event::poll(Duration::from_millis(10)) {
        Ok(true) => {
            let event = match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match mode {
                    AppMode::Normal => handle_normal_mode(key_event),
                    AppMode::Input => handle_input_mode(key_event),
                    AppMode::Command => handle_command_mode(key_event),
                },
                _ => None,
            };
            debug!("Generating event. Mode: {mode:?}, Event: {event:?}");
            Ok(event)
        }
        _ => Ok(None),
    }
}

fn handle_normal_mode(event: KeyEvent) -> Option<AppEvent> {
    let code = event.code;
    let modifiers = event.modifiers;
    match (code, modifiers) {
        (KeyCode::Enter, _) => Some(AppEvent::ChangeMode(AppMode::Input)),
        (KeyCode::Char('q'), _) => Some(AppEvent::Exit),
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => Some(AppEvent::Load),
        (KeyCode::Char('K'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollY(-1)),
        (KeyCode::Char('J'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollY(1)),
        (KeyCode::Char('H'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollX(-1)),
        (KeyCode::Char('L'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollX(1)),
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(AppEvent::CursorY(-1)),
        (KeyCode::Down | KeyCode::Char('j'), _) => Some(AppEvent::CursorY(1)),
        _ => None,
    }
}

fn handle_input_mode(event: KeyEvent) -> Option<AppEvent> {
    let code = event.code;
    match code {
        KeyCode::Esc => Some(AppEvent::ChangeMode(AppMode::Normal)),
        KeyCode::Enter => Some(AppEvent::Submit),
        _ => None,
    }
}

fn handle_command_mode(_event: KeyEvent) -> Option<AppEvent> {
    None
}
