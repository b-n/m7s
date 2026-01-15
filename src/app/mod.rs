#[allow(clippy::module_inception)]
mod app;
mod components;
mod error;
mod event;
mod file;
mod traits;

pub use app::{App, AppState};
pub use error::AppError;
pub use traits::AppComponent;

use event::{AppEvent, Delta};
use file::File;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum AppMode {
    #[default]
    Normal,
    Input,
    Command,
}

impl AppMode {
    fn display_text(&self) -> &str {
        match self {
            AppMode::Normal => "NORMAL",
            AppMode::Input => "INPUT",
            AppMode::Command => "COMMAND",
        }
    }
}
