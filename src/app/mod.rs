mod components;
mod error;
mod event;
mod file;
mod traits;
mod ui;

pub use error::AppError;
pub use traits::AppComponent;
pub use ui::App;

use event::{AppEvent, Delta};
use file::File;

#[derive(Default, Debug, Clone)]
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
