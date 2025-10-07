use log::debug;
use openapiv3::OpenAPI;
use serde::Deserialize;
use std::collections::HashMap;

use super::{ApiGroup, Error};

#[derive(Deserialize, Debug)]
struct PathSpec {
    #[serde(rename = "serverRelativeURL")]
    server_relative_url: String,
}

#[derive(Deserialize, Debug)]
pub struct RootSpec {
    paths: HashMap<String, PathSpec>,
}

impl From<&bytes::Bytes> for RootSpec {
    fn from(b: &bytes::Bytes) -> Self {
        serde_json::from_slice(b).unwrap()
    }
}

impl RootSpec {
    pub fn get_group_path(&self, group: &ApiGroup) -> Option<&str> {
        debug!("Getting path for group: {group}");
        let path = match group {
            ApiGroup::Core(v) => match v.as_str() {
                "" => "api".to_string(),
                v => format!("api/{v}"),
            },
            ApiGroup::Named(g, v) => match (g.as_str(), v.as_str()) {
                ("GROUPS", "") => "apis".to_string(),
                (g, "") => format!("apis/{g}"),
                (g, v) => format!("apis/{g}/{v}"),
            },
        };
        self.paths
            .get(&path)
            .map(|p| p.server_relative_url.as_str())
    }
}

pub struct QueryPath {
    key: String,
    child: Option<Box<QueryPath>>,
}

impl std::fmt::Display for QueryPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.child {
            Some(child) => write!(f, "{}.{}", self.key, child),
            None => write!(f, "{}", self.key),
        }
    }
}

impl QueryPath {
    pub fn new(key: &str) -> Self {
        QueryPath {
            key: key.to_string(),
            child: None,
        }
    }

    pub fn with_parent(self, parent: &str) -> Self {
        Self {
            key: parent.to_string(),
            child: Some(Box::new(self)),
        }
    }
}

#[derive(Debug)]
pub struct GroupSpec {
    group: ApiGroup,
    openapi: OpenAPI,
}

impl GroupSpec {
    pub fn new(group: ApiGroup, openapi: OpenAPI) -> Self {
        GroupSpec { group, openapi }
    }
}

impl GroupSpec {
    pub fn get_kind_path(
        &self,
        kind: &str,
        path: &QueryPath,
    ) -> Result<openapiv3::ReferenceOr<openapiv3::Schema>, Error> {
        debug!("Getting spec for kind {kind} at path {path}");

        let spec_name = format!("{}.{kind}", self.group.to_kube_group());
        let schemas = self
            .openapi
            .components
            .as_ref()
            .ok_or(Error::InvalidComponentsTree)?
            .schemas
            .get(&spec_name)
            .ok_or(Error::SpecNotFound(spec_name))?;

        // TODO: Need to walk the schema based on the query path

        Ok(schemas.clone())
    }

    #[allow(clippy::unused_self)]
    fn _walk_path(&self, _schema: openapiv3::Schema, _path: QueryPath) -> String {
        // TODO: Recurse through the schema to deliver some options
        // Return type should be Vec<SelectOption>
        "testing".to_string()
    }
}
