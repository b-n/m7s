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
}

fn ancestor_not_kind(node: SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    if node.kind() == kind {
        let parent = node
            .parent()
            .expect("All nodes should have parents")
            .clone();
        return ancestor_not_kind(parent, kind);
    }
    Some(node)
}

fn is_selectable_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::BLOCK_MAP_KEY | SyntaxKind::BLOCK_MAP_VALUE | SyntaxKind::DOCUMENT
    )
}

impl FileLine {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            preceding_whitespace: None,
            trailing_whitespace: None,
        }
    }

    fn render(&self, current_line: usize, cursor: (usize, usize), ast: &SyntaxNode) -> Line<'_> {
        let mut spans = Vec::new();

        if let Some(ws) = &self.preceding_whitespace {
            spans.push(Span::from(ws));
        }

        let mut selectable_values = 0;

        for (parent, tokens) in &self.tokens {
            let parent_kind = ancestor_not_kind(parent.to_node(ast), SyntaxKind::FLOW)
                .unwrap()
                .kind();

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
            debug!("Rendering span: {span:?}, Parent kind: {parent_kind:?}");

            // Apply styles
            let mut span = match parent_kind {
                SyntaxKind::BLOCK_MAP_KEY => span.style(Style::default().bold().fg(Color::Yellow)),
                _ => span,
            };

            // Highlight if needed
            if is_selectable_kind(parent_kind) {
                if cursor.0 == current_line && cursor.1 == selectable_values {
                    span = span.reversed();
                }
                selectable_values += 1;
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

fn tree_to_lines(tree: &SyntaxNode) -> Vec<FileLine> {
    let mut lines = Vec::new();
    let mut active_line = FileLine::new();

    let mut last_node = SyntaxNodePtr::new(tree);

    let mut token_buffer: Vec<SyntaxToken> = Vec::new();

    for event in tree.preorder_with_tokens() {
        match event {
            WalkEvent::Enter(element) => match element {
                NodeOrToken::Node(node) => {
                    debug!("++node: {node:?}");
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
                                active_line.trailing_whitespace = Some(ws.to_string());
                            }
                            // We then process all remaining parts. Each part drops any whitespace
                            // into a new line
                            for line in split_newlines {
                                // Store the active token
                                active_line.tokens.push((last_node, token_buffer.clone()));
                                lines.push(active_line);

                                token_buffer.clear();
                                // Note, there is no need to store the extra whitespace as it's
                                // own token, since we split it and add to preceding_whitespace
                                active_line = FileLine::new();
                                active_line.preceding_whitespace = Some(line.to_string());
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

                    // Move the last_node up the tree when exiting a node
                    if let Some(parent) = node.parent() {
                        last_node = SyntaxNodePtr::new(&parent);
                    }
                }
                NodeOrToken::Token(token) => {
                    debug!("--token {:?}", token.kind());
                    // Whitespace flushes itself, so we can skip it
                    if token.kind() == SyntaxKind::WHITESPACE {
                        // Whitespace tokens are handled on enter
                        continue;
                    }
                    if !token_buffer.is_empty() {
                        active_line.tokens.push((last_node, token_buffer.clone()));
                        token_buffer.clear();
                    }
                }
            },
        }
    }

    lines
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
            .map(|(i, line)| line.render(i, cursor, &self.ast))
            .collect();

        (lines, self.max_width)
    }

    pub fn info(&self, _cursor: (usize, usize)) {
        debug!("File path: {:?}", self.path);
    }
}
