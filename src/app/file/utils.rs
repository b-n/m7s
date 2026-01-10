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

fn whitespace_newlines(token: &SyntaxToken) -> Option<usize> {
    if token.kind() != SyntaxKind::WHITESPACE {
        return None;
    }
    let text = token.text();
    let without_newlines = text.replace('\n', "");
    Some(text.len() - without_newlines.len())
}

fn first_selectable_in_line(token: &SyntaxToken) -> SyntaxToken {
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

// Handles vertical movements from a token
fn selectable_y(token: &SyntaxToken, dir: &Direction) -> SyntaxToken {
    let mut selected = token.clone();
    let mut newlines = 0;
    let next_token = |token: SyntaxToken| -> Option<SyntaxToken> {
        match dir {
            Direction::Up(_) => token.prev_token(),
            Direction::Down(_) => token.next_token(),
            _ => unreachable!(),
        }
    };

    let target_newlines = match dir {
        Direction::Up(n) | Direction::Down(n) => *n,
        _ => unreachable!(),
    };

    // Move to the token that is at least target_newlines away
    while let Some(ref next) = next_token(selected.clone()) {
        if newlines >= target_newlines {
            break;
        }
        if let Some(n) = whitespace_newlines(next) {
            newlines += n;
        }
        selected = next.clone();
    }

    // Shortcircuit if no newlines were found
    if newlines == 0 {
        return selected;
    }

    // If we landed on a selectable token, return it
    if selectable_kind(selected.kind()) {
        return first_selectable_in_line(&selected);
    }

    // Otherwise, find the first selectable token we can find
    while let Some(ref next) = next_token(selected.clone()) {
        if selectable_kind(next.kind()) {
            selected = next.clone();
            break;
        }
        selected = next.clone();
    }

    // Fringe case, might have moved too far. Need to go backwards until we find something
    if !selectable_kind(selected.kind()) {
        while let Some(ref prev) = match dir {
            Direction::Up(_) => selected.next_token(),
            Direction::Down(_) => selected.prev_token(),
            _ => unreachable!(),
        } {
            if selectable_kind(prev.kind()) {
                selected = prev.clone();
                break;
            }
            selected = prev.clone();
        }
    }
    first_selectable_in_line(&selected)
}

// Handles horizontal movements from a token
fn selectable_x(token: &SyntaxToken, dir: &Direction) -> SyntaxToken {
    let mut selected = token.clone();
    let next_token = |token: &SyntaxToken| -> Option<SyntaxToken> {
        match dir {
            Direction::Left(_) => token.prev_token(),
            Direction::Right(_) => token.next_token(),
            _ => unreachable!(),
        }
    };
    let mut tokens = 0;
    let total_tokens = match dir {
        Direction::Left(n) | Direction::Right(n) => *n,
        _ => unreachable!(),
    };

    while let Some(ref next) = next_token(&selected) {
        if selectable_kind(next.kind()) {
            selected = next.clone();
            tokens += 1;
            if tokens >= total_tokens {
                break;
            }
        }

        if let Some(n) = whitespace_newlines(next)
            && n > 0
        {
            break;
        }

        selected = next.clone();
    }

    // Went nowhere, return the original token
    if tokens == 0 {
        return token.clone();
    }

    selected
}

// Assumption: The current token is always selectable
pub(crate) fn selectable_token_in_direction(token: &SyntaxToken, dir: &Direction) -> SyntaxToken {
    match dir {
        Direction::Up(n) | Direction::Down(n) | Direction::Left(n) | Direction::Right(n)
            if *n == 0 =>
        {
            token.clone()
        }
        Direction::Up(_) | Direction::Down(_) => selectable_y(token, dir),
        Direction::Left(_) | Direction::Right(_) => selectable_x(token, dir),
    }
}
