use std::error::Error as _;

use crate::api_client::Error as ApiError;
use crate::app::AppError;
use crate::config::ConfigError;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum Error {
    #[error("ConfigError")]
    ConfigError(#[from] ConfigError),
    #[error("AppError")]
    ApplicationError(#[from] AppError),
    #[error("ApiError")]
    ApiError(#[from] ApiError),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")?;
        if let Some(e) = self.source() {
            writeln!(f, "  Caused by: {e:?} - {e}")?;
        }
        Ok(())
    }
}
