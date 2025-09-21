#[derive(thiserror::Error)]
pub enum AppError {
    #[error("Terminal already initialized")]
    AlreadyInitialized,
    #[error("Terminal not initialized")]
    NotInitialized,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")
    }
}
