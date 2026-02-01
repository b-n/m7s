#[derive(thiserror::Error)]
pub enum AppError {
    #[error("Terminal already initialized")]
    AlreadyInitialized,
    #[error("Terminal not initialized")]
    NotInitialized,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("File error: {0}")]
    FileError(#[from] super::file::Error),
    #[error("Sending error: {0}")]
    SendError(#[from] std::sync::mpsc::SendError<crate::app::AppEvent>),
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")
    }
}
