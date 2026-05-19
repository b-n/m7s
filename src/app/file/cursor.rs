use super::{SyntaxNode, SyntaxNodePtr, TokenAtOffset};
use rowan::{NodeOrToken, TextSize, WalkEvent};

use super::NodeInfo;

#[derive(Debug, Default)]
pub struct Cursor {
    start: u32,
    end: u32,
    pub syntax_node: Option<SyntaxNodePtr>,
    pub node_info: Option<NodeInfo>,
}

impl Cursor {
    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn end(&self) -> u32 {
        self.end
    }

    pub fn line(&self) -> usize {
        self.node_info.as_ref().map_or(0, |info| info.line.start)
    }
}

impl From<(SyntaxNode, NodeInfo)> for Cursor {
    fn from((node, info): (SyntaxNode, NodeInfo)) -> Self {
        let text_range = node.text_range();
        Self {
            start: text_range.start().into(),
            end: text_range.end().into(),
            syntax_node: Some(SyntaxNodePtr::new(&node)),
            node_info: Some(info),
        }
    }
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
