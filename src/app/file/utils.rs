use rowan::{NodeOrToken, WalkEvent};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

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

pub(crate) fn selectable_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::COMMENT | SyntaxKind::BLOCK_SCALAR_TEXT | SyntaxKind::PLAIN_SCALAR
    )
}

pub(crate) fn node_dimensions(tree: &SyntaxNode) -> (usize, usize) {
    let mut max_width = 0;
    let mut line_count = 0;

    let mut pending_line = String::new();

    for event in tree.preorder_with_tokens() {
        if let WalkEvent::Enter(element) = &event
            && let NodeOrToken::Token(token) = element
        {
            let token_text = token.text();
            let mut split_newlines = token_text.split('\n').peekable();
            // Get the first element, it'll always have some value
            let tok = split_newlines
                .next()
                .expect("Tokens always have some content");

            pending_line.push_str(tok);

            for line in split_newlines {
                let line_len = pending_line.len();
                if line_len > max_width {
                    max_width = line_len;
                }
                line_count += 1;
                pending_line.clear();
                pending_line.push_str(line);
            }
        }
    }

    (line_count, max_width)
}

pub(crate) fn whitespace_newlines(token: &SyntaxToken) -> Option<usize> {
    if token.kind() != SyntaxKind::WHITESPACE {
        return None;
    }
    let text = token.text();
    let without_newlines = text.replace('\n', "");
    Some(text.len() - without_newlines.len())
}

pub(crate) fn first_selectable_in_line(token: &SyntaxToken) -> SyntaxToken {
    let mut selected = token.clone();

    let mut next_token = token.prev_token();
    while let Some(ref next) = next_token {
        if let Some(n) = whitespace_newlines(next)
            && n > 0
        {
            break;
        }
        if selectable_kind(next.kind()) {
            selected = next.clone();
        }

        next_token = next.prev_token();
    }

    selected
}

pub(crate) fn parent_node_until(start: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    if start.kind() == kind {
        return Some(start.clone());
    }
    match start.parent() {
        Some(p) => parent_node_until(&p, kind),
        None => None,
    }
}
