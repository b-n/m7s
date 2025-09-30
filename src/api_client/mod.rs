use futures::future::join_all;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as k8s_meta_v1;
use kube_client::{config::Config as KubeConfig, config::KubeConfigOptions, Client as KubeClient};
use std::collections::HashMap;

use crate::config::Config;
use crate::error::Error;

#[derive(Debug, Default)]
struct ApiClientCache {
    core_resources: HashMap<String, Vec<k8s_meta_v1::APIResource>>,
    groups: Vec<k8s_meta_v1::APIGroup>,
    group_resources: HashMap<String, k8s_meta_v1::APIResource>,
    initialized: bool,
}

pub struct ApiClient {
    client: KubeClient,
    cache: ApiClientCache,
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
    })
}

impl ApiClient {
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
}
