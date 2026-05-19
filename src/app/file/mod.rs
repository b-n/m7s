use log::debug;
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
};
use rowan::{
    NodeOrToken, TextRange, TokenAtOffset as RowanTokenAtOffset, WalkEvent,
    ast::SyntaxNodePtr as RowanSyntaxNodePtr,
};
use std::path::PathBuf;
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken, YamlLanguage};

pub mod cursor;
mod kube;
mod nav;
pub(crate) mod utils;

pub use cursor::Cursor;
use cursor::token_at_cursor;
//use kube::KubeDetails;
pub use nav::Direction;
use nav::node_in_direction;
use utils::{
    ancestor_not_kind, count_newlines, is_selectable_node, node_dimensions, selectable_kind,
};

pub(crate) type SyntaxNodePtr = RowanSyntaxNodePtr<YamlLanguage>;
pub(crate) type TokenAtOffset = RowanTokenAtOffset<SyntaxToken>;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("File path not found: {0}")]
    PathNotFound(PathBuf),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    YamlParseError(#[from] yaml_parser::SyntaxError),
}

pub type Range = std::ops::Range<usize>;

// TODO: Use a command  mode...
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub grandparent: Option<SyntaxNode>,
    pub parent: Option<SyntaxNode>,
    pub token: SyntaxToken,
    pub kind: SyntaxKind,
    pub line: Range,
    pub column: Range,
    pub indent: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub line: Range,
    pub column: Range,
    pub indent: String,
}

// TODO: Save file
#[derive(Debug, Clone)]
pub struct File {
    path: PathBuf,
    pub max_width: usize,
    pub line_count: usize,
    ast: SyntaxNode,
}

impl File {
    pub fn from_path(path: PathBuf) -> Result<Self, Error> {
        debug!("Loading file");
        if !path.exists() {
            Err(Error::PathNotFound(path.clone()))?;
        }
        let raw = std::fs::read_to_string(&path)?;

        let ast = yaml_parser::parse(&raw)?;

        let (line_count, max_width) = node_dimensions(&ast);

        Ok(Self {
            path,
            max_width,
            line_count,
            ast,
        })
    }

    /// Generate Ratatui lines from loaded file.
    ///
    /// `cursor` is the byte position in the file which is used for highlighting active elements.
    pub fn render<'a>(&'a self, cursor: &'a Cursor) -> (Vec<Line<'a>>, usize) {
        tree_to_lines(&self.ast, cursor)
    }

    pub fn first_selectable(&self) -> Cursor {
        let mut iter = self.ast.preorder();
        for event in &mut iter {
            if let WalkEvent::Enter(node) = event
                && is_selectable_node(&node)
            {
                let info = self.node_info(&node);
                return (node, info).into();
            }
        }
        Cursor::default()
    }

    /// Return byte position for the first selectable element on a specific line.
    /// Newline is defined by `\n` characters.
    /// If there are not selectable tokens on or after that line, it will search backwards until
    /// one is found.
    ///
    /// `line` is 0-indexed line number in the file.
    pub fn cursor_at_line(&self, line: usize) -> Cursor {
        let mut line_count = 0;
        let mut selected =
            token_at_cursor(&self.ast, 0).expect("All files have at least one token");

        let next = |token: &SyntaxToken| token.next_token();
        let prev = |token: &SyntaxToken| token.prev_token();

        // scroll to the line
        while let Some(ref token) = next(&selected) {
            let text = token.text();
            line_count += text.chars().filter(|c| *c == '\n').count();
            if line_count >= line {
                break;
            }

            selected = token.clone();
        }

        // Find the first selectable token
        while let Some(ref token) = next(&selected) {
            selected = token.clone();
            if selectable_kind(token.kind()) {
                log::info!("BREAK1");
                break;
            }
        }

        // It's possible we're at EOF and there are no selectable tokens. In that case, search
        // backwards.
        if !selectable_kind(selected.kind()) {
            while let Some(ref token) = prev(&selected) {
                selected = token.clone();
                if selectable_kind(token.kind()) {
                    log::info!("BREAK2");
                    break;
                }
            }
        }

        //selected.text_range().start().into()
        // TODO
        Cursor::default()
    }

    /// Given a current position in a file, find the next cursor position in the given direction.
    ///
    /// `y` directions seek based on the number of lines first, then try to find the first
    /// selectable token in the same direction. It's possible to return a position on a closer line
    /// than what is given if there are no selectable tokens in the direction specified
    /// `x` directions seek within the same line. If nothing is found in the direction, then the
    /// cursor position will not change.
    pub fn navigate(&self, current_cursor: &Cursor, direction: &Direction) -> Cursor {
        let current_node = current_cursor
            .syntax_node
            .map_or(self.ast.clone(), |node| node.to_node(&self.ast));

        log::info!("Current node: {current_node:?}");

        let new_node = node_in_direction(&current_node, direction);
        let node_info = self.node_info(&new_node);

        (new_node, node_info).into()
    }

    pub fn info(&self, _cursor: &Cursor) -> String {
        //let token = token_at_cursor(&self.ast, cursor).expect("Should always have a token");

        //let kube_details: KubeDetails = (&token).try_into().unwrap();

        //format!("Kubernetes Details: {kube_details:?}\nCursor: {cursor:?}\nToken: {token:?}")
        todo!()
    }

    pub fn node_info(&self, node: &SyntaxNode) -> NodeInfo {
        let mut line = 0;

        for event in self.ast.preorder_with_tokens() {
            if let WalkEvent::Enter(element) = event {
                match element {
                    NodeOrToken::Node(n) if &n == node => {
                        break;
                    }
                    NodeOrToken::Node(_) => {}
                    NodeOrToken::Token(t) => {
                        line += count_newlines(t.text());
                    }
                }
            }
        }

        // Properties:
        // - indent
        // - line, column

        match node.kind() {
            SyntaxKind::BLOCK_SCALAR => {
                let end = line + count_newlines(&node.text().to_string());
                NodeInfo {
                    indent: String::new(),
                    line: Range { start: line, end },
                    column: Range { start: 0, end: 0 },
                }
            }
            SyntaxKind::BLOCK_MAP_KEY => NodeInfo {
                indent: String::new(),
                line: Range {
                    start: line,
                    end: line,
                },
                column: Range { start: 0, end: 0 },
            },
            _ => {
                todo!()
            }
        }

        //let token = token_at_cursor(&self.ast, cursor).expect("Should always have a token");
        //let kind = token.kind();

        //let (line_info, col_info, indent) = token_position(&self.ast, &token);

        //TokenInfo {
        //    grandparent: token.parent().and_then(|parent| parent.parent()),
        //    parent: token.parent(),
        //    token,
        //    kind,
        //    line: line_info,
        //    column: col_info,
        //    indent,
        //}
    }

    /// Write the file to disk in the same location.
    ///
    /// Note: This function abi will change.
    pub fn write(&self) {
        // todo: make this call falliable and take a Option PathBuf for a new location if desired.
        let output = self.ast.to_string();

        std::fs::write(&self.path, output).unwrap();
    }
}

// Applies the given styling for a specific syntax kind.
fn styled_span(s: String, kind: SyntaxKind, active: bool) -> Span<'static> {
    let mut span = Span::from(s);

    let style = |kind: SyntaxKind| match kind {
        SyntaxKind::BLOCK_MAP_KEY => Style::default().bold().fg(ratatui::style::Color::Yellow),
        _ => Style::default(),
    };

    span = span.style(style(kind));

    // Change the highlight if this is the active element
    if active {
        span = span.reversed();
    }

    span
}

// This is the main render function. It walks the CST from rowan and returns Ratatui lines along
// with the maximum width of any line (this is helpful for x scrolling and saves recalculation).
fn tree_to_lines<'a>(tree: &'a SyntaxNode, cursor: &'a Cursor) -> (Vec<Line<'a>>, usize) {
    let active_node_range = cursor
        .syntax_node
        .map_or(TextRange::default(), |node| node.to_node(tree).text_range());

    let mut lines = Vec::new();
    let mut max_width = 0;

    let mut pending_line = vec![];
    let mut last_node = None;
    let mut indent = 0;
    let s = ' ';

    for event in tree.preorder_with_tokens() {
        match event {
            WalkEvent::Enter(element) => match element {
                NodeOrToken::Node(node) => {
                    debug!("{s:indent$}[n+] {node:?}");
                    last_node = Some(SyntaxNodePtr::new(&node));
                    indent += 1;
                }
                NodeOrToken::Token(token) => {
                    debug!("{s:indent$}[t+] {token:?} {:?}", token.text());

                    let active_token = active_node_range.contains_range(token.text_range());

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
                    indent += 1;
                }
            },
            WalkEvent::Leave(element) => match element {
                NodeOrToken::Node(node) => {
                    indent -= 1;
                    debug!("{s:indent$}[n-] {node:?}");
                    last_node = if let Some(parent) = node.parent() {
                        Some(SyntaxNodePtr::new(&parent))
                    } else {
                        None
                    };
                }
                NodeOrToken::Token(token) => {
                    indent -= 1;
                    debug!("{s:indent$}[t-] {:?}", token.kind());
                }
            },
        }
    }

    lines.push(Line::from(pending_line.clone()));
    pending_line.clear();

    (lines, max_width)
}
