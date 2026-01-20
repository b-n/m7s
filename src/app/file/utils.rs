use rowan::{NodeOrToken, WalkEvent};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

use super::Range;

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
        SyntaxKind::COMMENT
            | SyntaxKind::BLOCK_SCALAR_TEXT
            | SyntaxKind::PLAIN_SCALAR
            | SyntaxKind::DOUBLE_QUOTED_SCALAR
            | SyntaxKind::SINGLE_QUOTED_SCALAR
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

fn count_whitespace_newlines(text: &str) -> usize {
    text.chars().filter(|&c| c == '\n').count()
}

pub(crate) fn whitespace_newlines(token: &SyntaxToken) -> Option<usize> {
    if token.kind() != SyntaxKind::WHITESPACE {
        return None;
    }
    Some(count_whitespace_newlines(token.text()))
    //let text = token.text();
    //let without_newlines = text.replace('\n', "");
    //Some(text.len() - without_newlines.len())
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

// Takes a token in a given tree
// Returns a tuple of (line_range, column_range, indent_string)
pub(crate) fn token_position(
    tree: &SyntaxNode,
    search_token: &SyntaxToken,
) -> (Range, Range, String) {
    // Need to walk the tree up to the current token to count the lines.
    // For calculating the indent, we use the last whitespace token seen (or 0), and collect all
    // whitespace characters after the last newline
    // For columns, we need to keep track of all text on the current line that has been seen after
    // the last newline. The range is then the max width of the token plus the start position.

    let mut line = 0;
    let mut last_whitespace: Option<SyntaxToken> = None;
    let mut token_x = 0;

    for event in tree.preorder_with_tokens() {
        if let WalkEvent::Enter(element) = &event
            && let NodeOrToken::Token(token) = element
        {
            if token == search_token {
                break;
            }

            let text = token.text();
            let newlines = count_whitespace_newlines(text);

            // We only want whitespace tokens containing newlines
            if token.kind() == SyntaxKind::WHITESPACE && newlines > 0 {
                last_whitespace = Some(token.clone());
            }

            if newlines > 0 {
                token_x = text
                    .rfind('\n')
                    .map_or(text.len(), |pos| text.len() - pos - 1);
            } else {
                token_x += text.len();
            }

            line += newlines;
        }
    }

    let search_token_text = search_token.text();

    let line_range = Range {
        start: line,
        end: line + count_whitespace_newlines(search_token_text),
    };

    let indent = last_whitespace.map_or(String::new(), |ws| {
        let text = ws.text();
        let last_newline_pos = text.rfind('\n').map_or(0, |pos| pos + 1);
        text[last_newline_pos..].to_string()
    });

    let max_width = search_token_text
        .split('\n')
        .map(str::len)
        .max()
        .unwrap_or(0);

    let column_range = Range {
        start: token_x,
        end: token_x + max_width,
    };

    (line_range, column_range, indent)
}
