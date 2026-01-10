use log::debug;
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
};
use rowan::{
    ast::SyntaxNodePtr as RowanSyntaxNodePtr, NodeOrToken, TextSize,
    TokenAtOffset as RowanTokenAtOffset, WalkEvent,
};
use std::path::PathBuf;
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken, YamlLanguage};

mod utils;

pub(crate) type SyntaxNodePtr = RowanSyntaxNodePtr<YamlLanguage>;
pub(crate) type TokenAtOffset = RowanTokenAtOffset<SyntaxToken>;

use utils::{
    ancestor_not_kind, line_at_cursor, node_dimensions, selectable_kind,
    selectable_token_in_direction, token_at_cursor,
};

// TODO: Save file
#[derive(Debug, Clone)]
pub struct File {
    path: PathBuf,
    pub max_width: usize,
    pub line_count: usize,
    ast: SyntaxNode,
}

#[derive(Debug)]
pub enum Direction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
}

impl File {
    pub fn from_path(path: PathBuf) -> Self {
        debug!("Loading file");
        //TODO: Make this falliable
        let raw = std::fs::read_to_string(&path).unwrap();

        let ast = yaml_parser::parse(&raw).unwrap();

        let (line_count, max_width) = node_dimensions(&ast);

        Self {
            path,
            max_width,
            line_count,
            ast,
        }
    }

    pub fn render(&self, cursor: usize) -> (Vec<Line<'_>>, usize) {
        tree_to_lines(&self.ast, cursor.try_into().unwrap())
    }

    pub fn line_at_cursor(&self, cursor: u32) -> usize {
        line_at_cursor(&self.ast, cursor)
    }

    pub fn cursor_at_line(&self, line: usize) -> u32 {
        let mut line_count = 0;
        let mut selected = token_at_cursor(&self.ast, 0);

        // scroll to the line
        while let Some(ref next) = selected {
            let text = next.text();
            line_count += text.chars().filter(|c| *c == '\n').count();
            if line_count >= line {
                break;
            }

            selected = next.next_token();
        }

        // Find the first selectable token
        while let Some(ref next) = selected {
            if selectable_kind(next.kind()) {
                break;
            }
            selected = next.next_token();
        }

        selected
            .expect("Should always have a token")
            .text_range()
            .start()
            .into()
    }

    // Takes the current cursor position, and returns the cursor position of the relevant next
    // token.
    pub fn navigate_dir(&self, current_cursor: u32, direction: &Direction) -> u32 {
        let current_token = token_at_cursor(&self.ast, current_cursor)
            .expect("Cursor should always be at a valid token");

        selectable_token_in_direction(&current_token, direction)
            .text_range()
            .start()
            .into()
    }

    pub fn info(&self, cursor: u32) {
        let token = token_at_cursor(&self.ast, cursor);

        debug!("Cursor: {cursor:?}");
        debug!("Token: {token:?}");
    }

    pub fn write(&self) {
        let output = self.ast.to_string();

        std::fs::write(&self.path, output).unwrap();
    }
}

fn style(kind: SyntaxKind) -> Style {
    match kind {
        SyntaxKind::BLOCK_MAP_KEY => Style::default().bold().fg(ratatui::style::Color::Yellow),
        _ => Style::default(),
    }
}

fn styled_span(s: String, kind: SyntaxKind, active: bool) -> Span<'static> {
    let mut span = Span::from(s);
    span = span.style(style(kind));
    if active {
        span = span.reversed();
    }
    span
}

fn tree_to_lines(tree: &SyntaxNode, cursor: u32) -> (Vec<Line<'_>>, usize) {
    let mut lines = Vec::new();
    let mut max_width = 0;

    let mut pending_line = vec![];
    let mut last_node = None;

    for event in tree.preorder_with_tokens() {
        match event {
            WalkEvent::Enter(element) => match element {
                NodeOrToken::Node(node) => {
                    debug!("++node: {node:?}");
                    last_node = Some(SyntaxNodePtr::new(&node));
                }
                NodeOrToken::Token(token) => {
                    debug!("++token: {token:?} {:?}", token.text());

                    let active_token = token.text_range().contains(TextSize::new(cursor));

                    let parent_kind = ancestor_not_kind(
                        last_node
                            .expect("Tokens always have parent Nodes")
                            .to_node(tree),
                        SyntaxKind::FLOW,
                    )
                    .unwrap()
                    .kind();

                    let mut split_newlines = token.text().split('\n').peekable();

                    // Get the first element, it'll always have some value
                    let tok = split_newlines
                        .next()
                        .expect("Whitespace elements should always have some value");
                    pending_line.push(styled_span(tok.to_string(), parent_kind, active_token));

                    for line in split_newlines {
                        let line_len = pending_line.len();
                        if line_len > max_width {
                            max_width = line_len;
                        }
                        lines.push(Line::from(pending_line.clone()));
                        pending_line.clear();
                        pending_line.push(styled_span(line.to_string(), parent_kind, active_token));
                    }
                }
            },
            WalkEvent::Leave(element) => match element {
                NodeOrToken::Node(node) => {
                    debug!("--node {node:?}");
                    last_node = if let Some(parent) = node.parent() {
                        Some(SyntaxNodePtr::new(&parent))
                    } else {
                        None
                    };
                }
                NodeOrToken::Token(token) => {
                    debug!("--token {:?}", token.kind());
                }
            },
        }
    }

    lines.push(Line::from(pending_line.clone()));
    pending_line.clear();

    (lines, max_width)
}
