use http::Request;
use kube_client::{
    client::Body as KubeBody,
    config::{Config as KubeConfig, KubeConfigOptions},
    Client as KubeClient,
};
use log::debug;
use openapiv3::OpenAPI;

use crate::config::Config;

mod enums;
mod error;
mod spec;

pub use enums::ApiGroup;
pub use error::Error;
use spec::GroupSpec;
pub use spec::QueryPath;

pub struct ApiClient {
    client: KubeClient,
    response_cache: std::collections::HashMap<String, bytes::Bytes>,
}

pub async fn from_config(config: &Config) -> Result<ApiClient, Error> {
    let kube_config_options = KubeConfigOptions {
        context: Some(config.context.clone()),
        ..KubeConfigOptions::default()
    };

    let kube_config =
        KubeConfig::from_custom_kubeconfig(config.kube_config.clone(), &kube_config_options)
            .await?;

    let client = KubeClient::try_from(kube_config)?;

    Ok(ApiClient {
        client,
        response_cache: std::collections::HashMap::new(),
    })
}

impl ApiClient {
    pub async fn get_group_spec(&mut self, group: &ApiGroup) -> Result<GroupSpec, Error> {
        debug!("Getting spec for {group}");

        let root_spec: spec::RootSpec = self.get_root_spec().await?;

        let group_spec_uri = root_spec
            .get_group_path(group)
            .ok_or(Error::InvalidGroup(group.to_string()))?;

        debug!("Getting spec for {group}");
        let response = self.get_cached(group_spec_uri).await?;
        let openapi: OpenAPI = serde_json::from_slice(response)?;

        Ok(GroupSpec::new(group.clone(), openapi))
    }

    async fn get_root_spec(&mut self) -> Result<spec::RootSpec, Error> {
        let response = self.get_cached("/openapi/v3").await?;
        Ok(response.into())
    }

    async fn get_cached(&mut self, uri: &str) -> Result<&bytes::Bytes, Error> {
        if let std::collections::hash_map::Entry::Vacant(entry) =
            self.response_cache.entry(uri.to_string())
        {
            let request = Request::builder()
                .method("GET")
                .uri(uri)
                .body(KubeBody::empty())?;

            let bytes = self
                .client
                .send(request)
                .await?
                .into_body()
                .collect_bytes()
                .await?;

            entry.insert(bytes);
        }

        Ok(self.response_cache.get(uri).unwrap())
    }
}
