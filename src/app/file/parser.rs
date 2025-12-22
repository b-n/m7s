use super::utils::{ancestor_not_kind, is_selectable_kind};
use log::debug;
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};
use rowan::{ast::SyntaxNodePtr as RowanSyntaxNodePtr, NodeOrToken, WalkEvent};
use std::path::PathBuf;
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken, YamlLanguage};

type SyntaxNodePtr = RowanSyntaxNodePtr<YamlLanguage>;

#[derive(Debug, Clone)]
struct FileLine {
    preceding_whitespace: Option<String>,
    tokens: Vec<(SyntaxNodePtr, Vec<SyntaxToken>)>,
    trailing_whitespace: Option<String>,
    selectable_values: usize,
}

impl FileLine {
    fn new(preceding_whitespace: Option<String>) -> Self {
        Self {
            tokens: Vec::new(),
            preceding_whitespace,
            trailing_whitespace: None,
            selectable_values: 0,
        }
    }

    fn add_tokens(&mut self, parent: SyntaxNodePtr, tokens: Vec<SyntaxToken>, ast: &SyntaxNode) {
        let parent =
            SyntaxNodePtr::new(&ancestor_not_kind(parent.to_node(ast), SyntaxKind::FLOW).unwrap());

        if is_selectable_kind(parent.kind()) {
            self.selectable_values += 1;
        }

        self.tokens.push((parent, tokens));
    }

    fn render(&self, current_line: usize, cursor: (usize, usize)) -> Line<'_> {
        let mut spans = Vec::new();

        if let Some(ws) = &self.preceding_whitespace {
            spans.push(Span::from(ws));
        }

        let mut selectable = 0;

        for (parent, tokens) in &self.tokens {
            let parent_kind = parent.kind();

            let mut s = String::new();
            for token in tokens {
                // Generate text
                let text = token.text();
                s += text;
                if matches!(token.kind(), SyntaxKind::COLON | SyntaxKind::MINUS) {
                    s += " ";
                }
            }
            let span = Span::from(s);

            // Apply styles
            let mut span = match parent_kind {
                SyntaxKind::BLOCK_MAP_KEY => span.style(Style::default().bold().fg(Color::Yellow)),
                _ => span,
            };

            // Highlight if needed
            if is_selectable_kind(parent_kind) {
                if cursor.0 == current_line {
                    // Current line and value is selectable
                    if cursor.1 == selectable {
                        span = span.reversed();
                    }
                    // Ugly, but a fringe case, select the last selectable value
                    if cursor.1 >= self.selectable_values
                        && selectable + 1 == self.selectable_values
                    {
                        span = span.reversed();
                    }
                }
                selectable += 1;
            }

            // Add to line
            spans.push(span);
        }

        if let Some(ws) = &self.trailing_whitespace {
            spans.push(Span::from(ws));
        }

        Line::from(spans)
    }
}

// TODO: Save file
#[derive(Debug)]
pub struct File {
    path: PathBuf,
    pub max_width: usize,
    pub line_count: usize,
    raw: String,
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

        // TODO: Maybe a better way to handle this?
        //let max_width = lines.max_width();
        let max_width = 100;
        let line_count = lines.len();

        Self {
            path,
            max_width,
            line_count,
            raw,
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

    pub fn info(&self, _cursor: (usize, usize)) {
        debug!("File path: {:?}", self.path);
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

                            if split_newlines.peek().is_none() {
                                // No newlines, just whitespace
                                token_buffer.push(token.clone());
                                continue;
                            }

                            // There is a newline, so we store the first part of the split in the
                            // current line (could be trailing whitespace)
                            if let Some(ws) = split_newlines.next() {
                                pending_line.trailing_whitespace = Some(ws.to_string());
                            }
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
