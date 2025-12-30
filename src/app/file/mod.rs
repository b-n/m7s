use log::debug;
use ratatui::text::Line;
use rowan::{ast::SyntaxNodePtr as RowanSyntaxNodePtr, NodeOrToken, WalkEvent};
use std::path::PathBuf;
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken, YamlLanguage};

mod line;
mod utils;

type SyntaxNodePtr = RowanSyntaxNodePtr<YamlLanguage>;

pub use line::FileLine;

// TODO: Save file
#[derive(Debug)]
pub struct File {
    path: PathBuf,
    pub max_width: usize,
    pub line_count: usize,
    lines: Vec<FileLine>,
    ast: SyntaxNode,
}

impl File {
    pub fn from_path(path: PathBuf) -> Self {
        debug!("Loading file");
        //TODO: Make this falliable
        let raw = std::fs::read_to_string(&path).unwrap();

        let ast = yaml_parser::parse(&raw).unwrap();
        let lines = tree_to_lines(&ast);

        let max_width = lines.iter().map(FileLine::length).max().unwrap_or(0);
        let line_count = lines.len();

        Self {
            path,
            max_width,
            line_count,
            lines,
            ast,
        }
    }

    pub fn render(&self, cursor: (usize, usize)) -> (Vec<Line<'_>>, usize) {
        let lines = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| line.render(i, cursor))
            .collect();

        (lines, self.max_width)
    }

    pub fn info(&self, cursor: (usize, usize)) {
        let token: Option<SyntaxNodePtr> = self
            .lines
            .get(cursor.0)
            .and_then(|line| line.selectable_token_at(cursor.1));

        debug!("Cursor: {cursor:?}");
        debug!("Token: {token:?}");
    }

    pub fn write(&self) {
        let output = self.ast.to_string();

        std::fs::write(&self.path, output).unwrap();
    }
}

// Processing logic:
// - Each node can have child tokens and child nodes
// - Tokens are collected under their parent node
// - Nodes and Tokens are collected by newlines
// - Newlines only exist in whitespace tokens
//
// Important:
// - A line can have many collections of tokens.
// - Each collection of tokens should be associated with it's parent node
//
// Therefore:
// - We traverse the tree in order
// - For each token, it is added to a buffer
// - Since a node can have both nodes and tokens as children, the buffer needs to be flushed on
//   both entry and exit of nodes
// - If a node is entered or exited:
//   - Flush the token buffer to the `pending_line` with the previous last known node
//   - Update the last known node. Entry: current, Exit: parent of current
fn tree_to_lines(tree: &SyntaxNode) -> Vec<FileLine> {
    let mut lines = Vec::new();

    let mut pending_line = FileLine::new(None);
    let mut last_node = SyntaxNodePtr::new(tree);
    let mut token_buffer: Vec<SyntaxToken> = Vec::new();

    for event in tree.preorder_with_tokens() {
        match event {
            WalkEvent::Enter(element) => match element {
                NodeOrToken::Node(node) => {
                    debug!("++node: {node:?}");
                    // A node may have tokens and children, so we need to flush the buffered tokens
                    // before moving down the tree
                    if !token_buffer.is_empty() {
                        pending_line.add_tokens(last_node, token_buffer.clone(), tree);
                        token_buffer.clear();
                    }

                    // Moving the parent down the tree
                    last_node = SyntaxNodePtr::new(&node);
                }
                NodeOrToken::Token(token) => {
                    debug!("++token: {token:?} {:?}", token.text());
                    match token.kind() {
                        SyntaxKind::WHITESPACE => {
                            let mut split_newlines = token.text().split('\n').peekable();

                            // Get the first element, it'll always have some value
                            let ws = split_newlines
                                .next()
                                .expect("Whitespace elements should always have some value");

                            if split_newlines.peek().is_none() {
                                // No newlines, just whitespace
                                token_buffer.push(token.clone());
                                continue;
                            }

                            // There is a newline, so we store the first part of the split in the
                            // current line (could be trailing whitespace)
                            pending_line.trailing_whitespace(Some(ws.to_string()));

                            // We then process all remaining parts. Each part drops any whitespace
                            // into a new line
                            for line in split_newlines {
                                // Store the active token
                                pending_line.add_tokens(last_node, token_buffer.clone(), tree);
                                lines.push(pending_line);

                                token_buffer.clear();
                                // Note, there is no need to store the extra whitespace as it's
                                // own token, since we split it and add to preceding_whitespace
                                pending_line = FileLine::new(Some(line.to_string()));
                            }
                        }
                        _ => {
                            token_buffer.push(token.clone());
                        }
                    }
                }
            },
            WalkEvent::Leave(element) => match element {
                NodeOrToken::Node(node) => {
                    debug!("--node {node:?}");

                    if !token_buffer.is_empty() {
                        pending_line.add_tokens(last_node, token_buffer.clone(), tree);
                        token_buffer.clear();
                    }

                    // Move the last_node up the tree when exiting a node
                    if let Some(parent) = node.parent() {
                        last_node = SyntaxNodePtr::new(&parent);
                    }
                }
                NodeOrToken::Token(token) => {
                    debug!("--token {:?}", token.kind());
                }
            },
        }
    }

    lines
}
