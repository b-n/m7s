use kube_client::{config::KubeconfigError, error::Error as KubeError};
use std::error::Error as _;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum Error {
    #[error("KubeError")]
    KubeError(#[from] KubeError),
    #[error("KubeconfigError")]
    KubeconfigError(#[from] KubeconfigError),
    #[error("HttpError")]
    HttpError(#[from] http::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::error::Error),
    #[error("Invalid group path {0}")]
    InvalidGroup(String),
    #[error("All openapi specs require a components attribute")]
    InvalidComponentsTree,
    #[error("Could not find spec for {0}")]
    SpecNotFound(String),
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
