use crate::config::ConfigError;

#[derive(Debug)]
pub enum Error {
    ConfigError(ConfigError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConfigError(e) => write!(f, "Configuration error: {e}"),
        }
    }
}

impl From<ConfigError> for Error {
    fn from(e: ConfigError) -> Self {
        Error::ConfigError(e)
    }
}

impl std::error::Error for Error {}
