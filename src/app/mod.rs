mod error;
mod event;
mod ui;

pub use error::AppError;
use event::AppEvent;
pub use ui::App;

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
