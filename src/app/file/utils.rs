use yaml_parser::{SyntaxKind, SyntaxNode};

pub(crate) fn ancestor_not_kind(node: SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    if node.kind() == kind {
        let parent = node
            .parent()
            .expect("All nodes should have parents")
            .clone();
        return ancestor_not_kind(parent, kind);
    }
    Some(node)
}

pub(crate) fn is_selectable_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::BLOCK_MAP_KEY | SyntaxKind::BLOCK_MAP_VALUE | SyntaxKind::DOCUMENT
    )
}
