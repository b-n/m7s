use crate::api_client::Error as ApiError;
use crate::app::AppError;
use crate::config::ConfigError;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ConfigError")]
    ConfigError(#[from] ConfigError),
    #[error("AppError")]
    ApplicationError(#[from] AppError),
    #[error("ApiError")]
    ApiError(#[from] ApiError),
}
