use super::{SyntaxNode, TokenAtOffset};
use rowan::{NodeOrToken, TextSize, WalkEvent};

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
