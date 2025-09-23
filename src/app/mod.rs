mod error;
mod event;
mod file;
mod ui;

pub use error::AppError;
pub use ui::App;

use event::AppEvent;
use file::File;

#[derive(Default)]
enum AppMode {
    #[default]
    Normal,
    Input,
}

impl AppMode {
    fn display_text(&self) -> &str {
        match self {
            AppMode::Normal => "NORMAL",
            AppMode::Input => "INPUT",
        }
    }
}
