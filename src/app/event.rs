use log::debug;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use tokio::time::Duration;

use super::AppMode;

#[derive(Debug)]
pub enum Delta {
    Inc(usize),
    Dec(usize),
    Zero,
}

impl From<&Delta> for isize {
    fn from(delta: &Delta) -> Self {
        match delta {
            Delta::Inc(v) => 0isize.saturating_add_unsigned(*v),
            Delta::Dec(v) => 0isize.saturating_sub_unsigned(*v),
            Delta::Zero => 0,
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
    ChangeMode(AppMode),
    Exit,
    Submit,
    Load,
    CursorY(Delta),
    CursorX(Delta),
    ScrollX(Delta),
    ScrollY(Delta),
    TerminalResize,
    LoadSpec,
    Info,
    Write,
}

pub fn handle_event(mode: &AppMode) -> io::Result<Option<AppEvent>> {
    match event::poll(Duration::from_millis(10)) {
        Ok(true) => {
            let event = match event::read()? {
                Event::Resize(_, _) => Some(AppEvent::TerminalResize),
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
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(AppEvent::Write),
        (KeyCode::Char('s'), KeyModifiers::SHIFT) => Some(AppEvent::LoadSpec),
        (KeyCode::Char('K'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollY(Delta::Dec(1))),
        (KeyCode::Char('J'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollY(Delta::Inc(1))),
        (KeyCode::Char('H'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollX(Delta::Dec(1))),
        (KeyCode::Char('L'), KeyModifiers::SHIFT) => Some(AppEvent::ScrollX(Delta::Inc(1))),
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(AppEvent::CursorY(Delta::Dec(1))),
        (KeyCode::Down | KeyCode::Char('j'), _) => Some(AppEvent::CursorY(Delta::Inc(1))),
        (KeyCode::Left | KeyCode::Char('h'), _) => Some(AppEvent::CursorX(Delta::Dec(1))),
        (KeyCode::Right | KeyCode::Char('l'), _) => Some(AppEvent::CursorX(Delta::Inc(1))),
        (KeyCode::PageUp, KeyModifiers::SHIFT) => Some(AppEvent::ScrollX(Delta::Dec(10))),
        (KeyCode::PageDown, KeyModifiers::SHIFT) => Some(AppEvent::ScrollX(Delta::Inc(10))),
        (KeyCode::PageUp, _) => Some(AppEvent::ScrollY(Delta::Dec(10))),
        (KeyCode::PageDown, _) => Some(AppEvent::ScrollY(Delta::Inc(10))),
        (KeyCode::Char('i'), _) => Some(AppEvent::Info),
        _ => None,
    }
}

fn handle_input_mode(event: KeyEvent) -> Option<AppEvent> {
    let code = event.code;
    match code {
        KeyCode::Esc => Some(AppEvent::ChangeMode(AppMode::Normal)),
        KeyCode::Enter => Some(AppEvent::Submit),
        KeyCode::Char('i') => Some(AppEvent::Info),
        _ => None,
    }
}

fn handle_command_mode(_event: KeyEvent) -> Option<AppEvent> {
    None
}
