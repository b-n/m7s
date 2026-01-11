use yaml_parser::SyntaxToken;

use super::utils::{first_selectable_in_line, selectable_kind, whitespace_newlines};

#[derive(Debug)]
pub enum Direction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
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
