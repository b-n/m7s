use log::{debug, info};
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

mod cursor;
mod kube;
mod nav;
pub(crate) mod utils;

use cursor::{line_at_cursor, token_at_cursor};
use kube::KubeDetails;
use nav::selectable_token_in_direction;
pub use nav::Direction;
use utils::{ancestor_not_kind, node_dimensions, selectable_kind, token_position};

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

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: SyntaxToken,
    pub kind: SyntaxKind,
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
    pub fn render(&self, cursor: usize) -> (Vec<Line<'_>>, usize) {
        tree_to_lines(&self.ast, cursor.try_into().unwrap())
    }

    /// Get the line number for a specific byte position in the loaded file.
    ///
    /// `cursor` is the byte position in the file.
    pub fn line_at_cursor(&self, cursor: u32) -> usize {
        line_at_cursor(&self.ast, cursor)
    }

    /// Return byte position for the first selectable element on a specific line.
    /// Newline is defined by `\n` characters.
    /// If there are not selectable tokens on or after that line, it will search backwards until
    /// one is found.
    ///
    /// `line` is 0-indexed line number in the file.
    pub fn first_selectable_at_line(&self, line: usize) -> u32 {
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

        selected.text_range().start().into()
    }

    /// Given a current position in a file, find the next cursor position in the given direction.
    ///
    /// `y` directions seek based on the number of lines first, then try to find the first
    /// selectable token in the same direction. It's possible to return a position on a closer line
    /// than what is given if there are no selectable tokens in the direction specified
    /// `x` directions seek within the same line. If nothing is found in the direction, then the
    /// cursor position will not change.
    pub fn navigate_dir(&self, current_cursor: u32, direction: &Direction) -> u32 {
        let current_token = token_at_cursor(&self.ast, current_cursor)
            .expect("Cursor should always be at a valid token");

        selectable_token_in_direction(&current_token, direction)
            .text_range()
            .start()
            .into()
    }

    pub fn info(&self, cursor: u32) {
        let token = token_at_cursor(&self.ast, cursor).expect("Should always have a token");

        let kube_details: KubeDetails = (&token).try_into().unwrap();

        info!("Kubernetes Details: {kube_details:?}");
        info!("Cursor: {cursor:?}");
        info!("Token: {token:?}");
    }

    pub fn token_info_at_cursor(&self, cursor: u32) -> TokenInfo {
        let token = token_at_cursor(&self.ast, cursor).expect("Should always have a token");
        let kind = token.kind();

        let (line_info, col_info, indent) = token_position(&self.ast, &token);

        TokenInfo {
            token,
            kind,
            line: line_info,
            column: col_info,
            indent,
        }
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
