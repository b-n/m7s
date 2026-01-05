use rowan::{NodeOrToken, TextSize, WalkEvent};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

use super::{Direction, TokenAtOffset};

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

pub(crate) fn line_at_cursor(tree: &SyntaxNode, cursor: u32) -> usize {
    let mut line_count = 0;

    for event in tree.preorder_with_tokens() {
        if let WalkEvent::Enter(element) = &event
            && let NodeOrToken::Token(token) = element
        {
            if token.text_range().contains(TextSize::new(cursor)) {
                break;
            }
            let text = token.text();
            let without_newlines = text.replace('\n', "");
            let newlines = text.len() - without_newlines.len();

            line_count += newlines;
        }
    }

    line_count
}

pub(crate) fn token_at_cursor(
    tree: &SyntaxNode,
    cursor: u32,
) -> Option<rowan::SyntaxToken<yaml_parser::YamlLanguage>> {
    match tree.token_at_offset(TextSize::new(cursor)) {
        TokenAtOffset::Single(token) => Some(token),
        TokenAtOffset::Between(_, token) => {
            // Always favour the righthand token. Range ends are inclusive
            Some(token)
        }
        TokenAtOffset::None => None,
    }
}

fn token_is_newlined_whitespace(token: &SyntaxToken) -> bool {
    token.kind() == SyntaxKind::WHITESPACE && token.text().contains('\n')
}

fn first_selectable_in_line(token: &SyntaxToken) -> SyntaxToken {
    let mut selected = token.clone();

    let mut next_token = token.prev_token();
    while let Some(ref next) = next_token {
        if token_is_newlined_whitespace(next) {
            break;
        }
        if selectable_kind(next.kind()) {
            selected = next.clone();
        }

        next_token = next.prev_token();
    }

    selected
}

pub(crate) fn token_in_direction(token: &SyntaxToken, dir: &Direction) -> SyntaxToken {
    let mut selected = token.clone();
    match dir {
        Direction::Up(n) | Direction::Down(n) | Direction::Left(n) | Direction::Right(n)
            if *n == 0 => {}
        Direction::Up(n) => {
            let mut newlines = 0;
            let mut next_token = token.prev_token();
            while let Some(ref next) = next_token {
                if selectable_kind(next.kind()) {
                    selected = next.clone();
                }

                if newlines >= *n {
                    break;
                }

                if token_is_newlined_whitespace(next) {
                    newlines += 1;
                }

                next_token = next.prev_token();
            }

            // The above will find the last token in the line. That feels weird. Use the first
            // token instead
            selected = first_selectable_in_line(&selected);
        }
        Direction::Down(n) => {
            let mut newlines = 0;
            let mut next_token = token.next_token();
            while let Some(ref next) = next_token {
                if selectable_kind(next.kind()) {
                    selected = next.clone();
                }
                if newlines >= *n && selectable_kind(next.kind()) {
                    break;
                }
                if token_is_newlined_whitespace(next) {
                    newlines += 1;
                }

                next_token = next.next_token();
            }
            // If we didn't go anywhere, return the input value
            if newlines == 0 {
                return token.clone();
            }
        }
        Direction::Right(n) => {
            let mut tokens = 0;
            let mut next_token = token.next_token();
            while let Some(ref next) = next_token {
                if selectable_kind(next.kind()) {
                    selected = next.clone();
                    tokens += 1;
                    if tokens >= *n {
                        break;
                    }
                }

                if token_is_newlined_whitespace(next) {
                    break;
                }

                next_token = next.next_token();
            }
        }
        Direction::Left(n) => {
            let mut tokens = 0;
            let mut next_token = token.prev_token();
            while let Some(ref next) = next_token {
                if selectable_kind(next.kind()) {
                    selected = next.clone();
                    tokens += 1;
                    if tokens >= *n {
                        break;
                    }
                }

                if token_is_newlined_whitespace(next) {
                    break;
                }

                next_token = next.prev_token();
            }
        }
    }
    selected
}
