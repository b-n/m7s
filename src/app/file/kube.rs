use super::{SyntaxKind, SyntaxToken};
use crate::api_client::ApiGroup;
use std::collections::HashMap;
use yaml_parser::ast::{AstNode, BlockMapValue, Document};

use super::utils::parent_node_until;

#[derive(thiserror::Error, Debug)]
pub enum KubeDetailsError {
    //#[error("Current position is not within a valid Kubernets Manifest. Requires both `kind` and `apiVersion` fields.")]
    //NotKubernetesManifest,
    #[error("Syntax token has no parent syntax node.")]
    NoSyntaxNodeParent,
    #[error("Root syntax node is not a document.")]
    RootNodeNotDocument,
    #[error("Document node is not a block map.")]
    DocumentIsNotBlockMap,
    #[error("Document is missing either `kind` or `apiVersion` fields.")]
    DocumentMissingKindOrApiVersion,
}

#[derive(Debug, Clone)]
pub struct KubeDetails {
    kind: String,
    api_version: ApiGroup,
}

impl TryFrom<&SyntaxToken> for KubeDetails {
    type Error = KubeDetailsError;

    fn try_from(token: &SyntaxToken) -> Result<Self, Self::Error> {
        let root = token.parent().ok_or(KubeDetailsError::NoSyntaxNodeParent)?;

        let doc_node = parent_node_until(&root, SyntaxKind::DOCUMENT)
            .ok_or(KubeDetailsError::RootNodeNotDocument)?;

        let doc_entries = Document::cast(doc_node)
            .expect("Document node should always cast to Document AST")
            .block()
            .ok_or(KubeDetailsError::DocumentIsNotBlockMap)?
            .block_map()
            .ok_or(KubeDetailsError::DocumentIsNotBlockMap)?
            .entries()
            .filter_map(|entry| {
                entry
                    .key()
                    .map(|key| key.syntax().text().to_string())
                    .map(|key| (key, entry.value()))
            })
            .collect::<HashMap<String, Option<BlockMapValue>>>();

        let kind = doc_entries
            .get("kind")
            .and_then(|v| v.as_ref())
            .map(|value| value.syntax().text().to_string());

        let api_version = doc_entries
            .get("apiVersion")
            .and_then(|v| v.as_ref())
            .map(|value| value.syntax().text().to_string());

        match (api_version, kind) {
            (Some(api_version), Some(kind)) => Ok(KubeDetails {
                kind,
                api_version: (*api_version).into(),
            }),
            _ => Err(KubeDetailsError::DocumentMissingKindOrApiVersion),
        }
    }
}
