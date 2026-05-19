use yaml_parser::{SyntaxNode, SyntaxToken};

use super::utils::{count_newlines, is_selectable_node, whitespace_newlines};

#[derive(Debug)]
pub enum Direction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
}

fn selectable_node(node: &SyntaxNode) -> Option<SyntaxNode> {
    if is_selectable_node(node) {
        return Some(node.clone());
    }

    let mut target = node.clone();
    while let Some(parent) = target.parent() {
        if is_selectable_node(&parent) {
            return Some(parent);
        }
        target = parent;
    }
    None
}

// Handles vertical movements from a node
fn selectable_y(node: &SyntaxNode, dir: &Direction) -> SyntaxNode {
    // Get the first/last token in the node.
    let mut selected = match dir {
        Direction::Up(_) => node.first_token(),
        Direction::Down(_) => node.last_token(),
        _ => unreachable!(),
    }
    .expect("Should always have a token");

    // Start counting tokens in the direction needed
    let mut newlines = 0;
    let target_newlines = match dir {
        Direction::Up(n) | Direction::Down(n) => *n,
        _ => unreachable!(),
    };

    let next_token = |token: SyntaxToken| -> Option<SyntaxToken> {
        match dir {
            Direction::Up(_) => token.prev_token(),
            Direction::Down(_) => token.next_token(),
            _ => unreachable!(),
        }
    };

    // Manage a stack, just incase we need to backtrack
    let mut token_stack = vec![];
    while let Some(ref next) = next_token(selected.clone()) {
        token_stack.push(next.clone());
        if newlines >= target_newlines {
            break;
        }
        if let Some(n) = whitespace_newlines(next) {
            newlines += n;
        }
        selected = next.clone();
    }
    // Selected should now be X lines away

    // Shortcircuit if no newlines were found
    if newlines == 0 {
        return node.clone();
    }

    // If we found a selectable node, return it
    let parent_node = selected.parent().expect("All tokens should have a parent");
    if selectable_node(&parent_node).is_some() {
        return parent_node;
    }

    // Otherwise, keep going forward in the tokens until we find something
    while let Some(ref next) = next_token(selected.clone()) {
        let parent_node = next.parent().expect("All tokens should have a parent");
        if selectable_node(&parent_node).is_some() {
            return parent_node;
        }
        selected = next.clone();
    }

    // And in the case we didn't find anything, then return up the stack until we find something
    while let Some(ref next) = token_stack.pop() {
        log::info!("next next next: {next:?}");
        let parent_node = next.parent().expect("All tokens should have a parent");
        if selectable_node(&parent_node).is_some() {
            return parent_node;
        }
    }

    // And very lastly, if we found nothing, just return the input node
    node.clone()
}

// Handles horizontal movements from a token
fn selectable_x(node: &SyntaxNode, dir: &Direction) -> SyntaxNode {
    let mut selected = match dir {
        Direction::Left(_) => node.first_token(),
        Direction::Right(_) => node.last_token(),
        _ => unreachable!(),
    }
    .expect("Should always have a token");

    let next_token = |token: &SyntaxToken| -> Option<SyntaxToken> {
        match dir {
            Direction::Left(_) => token.prev_token(),
            Direction::Right(_) => token.next_token(),
            _ => unreachable!(),
        }
    };
    let mut selectable = 0;
    let total_selectable = match dir {
        Direction::Left(n) | Direction::Right(n) => *n,
        _ => unreachable!(),
    };

    while let Some(ref next) = next_token(&selected) {
        let parent_node = next.parent().expect("All tokens should have a parent");
        if selectable_node(&parent_node).is_some() {
            selected = match dir {
                Direction::Left(_) => parent_node.first_token(),
                Direction::Right(_) => parent_node.last_token(),
                _ => unreachable!(),
            }
            .expect("All parents has tokens");

            selectable += 1;
            if selectable >= total_selectable
                || count_newlines(&parent_node.text().to_string()) > 0
            {
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
    if selectable == 0 {
        return node.clone();
    }

    // Otherwise we can return the node attached to the token (if it is selectable)
    let parent_node = selected.parent().expect("All tokens should have a parent");
    if selectable_node(&parent_node).is_some() {
        return parent_node;
    }

    // Otherwise just return the original node
    node.clone()
}

// Assumption: The current token is always selectable
pub(crate) fn node_in_direction(node: &SyntaxNode, dir: &Direction) -> SyntaxNode {
    match dir {
        Direction::Up(n) | Direction::Down(n) | Direction::Left(n) | Direction::Right(n)
            if *n == 0 =>
        {
            node.clone()
        }
        Direction::Up(_) | Direction::Down(_) => selectable_y(node, dir),
        Direction::Left(_) | Direction::Right(_) => selectable_x(node, dir),
    }
}
