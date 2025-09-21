use kube_client::config::KubeconfigError;
use kube_client::error::Error as KubeError;
use std::error::Error as _;

use crate::app::AppError;
use crate::config::ConfigError;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum Error {
    #[error("ConfigError")]
    ConfigError(#[from] ConfigError),
    #[error("KubeError")]
    KubeError(#[from] KubeError),
    #[error("KubeconfigError")]
    KubeconfigError(#[from] KubeconfigError),
    #[error("AppError")]
    ApplicationError(#[from] AppError),
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
