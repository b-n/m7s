use futures::future::join_all;
use http::Request;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as k8s_meta_v1;
use kube_client::{
    client::Body as KubeBody,
    config::{Config as KubeConfig, KubeConfigOptions},
    Client as KubeClient,
};
use log::debug;
use openapiv3::OpenAPI;
use std::collections::HashMap;

use crate::config::Config;

mod enums;
mod error;
mod spec;

pub use enums::ApiGroup;
pub use error::Error;
use spec::GroupSpec;
pub use spec::QueryPath;

#[derive(Debug, Default)]
struct ApiClientCache {
    core_resources: HashMap<String, Vec<k8s_meta_v1::APIResource>>,
    group_resources: HashMap<String, k8s_meta_v1::APIResource>,
    initialized: bool,
}

pub struct ApiClient {
    client: KubeClient,
    cache: ApiClientCache,
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
        cache: ApiClientCache::default(),
        response_cache: std::collections::HashMap::new(),
    })
}

impl ApiClient {
    // TODO: Rethink the kind/group caching logic. Theory: people only care about kinds, not
    // groups. Have a look at kube_client::discovery since that might do it all for us.
    async fn populate_cache(&mut self) -> Result<(), Error> {
        if self.cache.initialized {
            return Ok(());
        }

        let core_versions = self.client.list_core_api_versions().await?.versions;

        self.cache.core_resources = join_all(core_versions.iter().map(|version| async {
            self.client
                .list_core_api_resources(version)
                .await
                .map(|res| (res.group_version, res.resources))
        }))
        .await
        // collect for resolving the results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        // collect for (k,v) into HashMap
        .into_iter()
        .collect();

        Ok(())
    }

    pub async fn get_kinds(&mut self) -> Result<Vec<String>, Error> {
        self.populate_cache().await?;

        let kinds = self
            .cache
            .core_resources
            .values()
            .flatten()
            .map(|res| res.kind.clone())
            .collect();

        Ok(kinds)
    }

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
