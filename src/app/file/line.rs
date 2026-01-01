use super::utils::{ancestor_not_kind, is_selectable_kind};
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};
use rowan::ast::SyntaxNodePtr as RowanSyntaxNodePtr;
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken, YamlLanguage};

type SyntaxNodePtr = RowanSyntaxNodePtr<YamlLanguage>;

#[derive(Debug, Clone)]
pub struct FileLine {
    preceding_whitespace: Option<String>,
    tokens: Vec<(SyntaxNodePtr, Vec<SyntaxToken>)>,
    trailing_whitespace: Option<String>,
    selectable: usize,
    length: usize,
}

impl FileLine {
    pub fn new(preceding_whitespace: Option<String>) -> Self {
        let length = preceding_whitespace.as_ref().map_or(0, String::len);

        Self {
            tokens: Vec::new(),
            preceding_whitespace,
            trailing_whitespace: None,
            selectable: 0,
            length,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn selectable(&self) -> usize {
        self.selectable
    }

    pub fn selectable_token_at(&self, index: usize) -> Option<SyntaxNodePtr> {
        let mut selectable = 0;

        for (parent, _) in &self.tokens {
            if is_selectable_kind(parent.kind()) {
                if selectable == index {
                    return Some(*parent);
                }
                selectable += 1;
            }
        }

        None
    }

    pub fn trailing_whitespace(&mut self, ws: Option<String>) {
        self.length += ws.as_ref().map_or(0, String::len);
        self.trailing_whitespace = ws;
    }

    pub fn add_tokens(
        &mut self,
        parent: SyntaxNodePtr,
        tokens: Vec<SyntaxToken>,
        ast: &SyntaxNode,
    ) {
        let parent =
            SyntaxNodePtr::new(&ancestor_not_kind(parent.to_node(ast), SyntaxKind::FLOW).unwrap());

        self.length += tokens.iter().fold(0, |acc, token| acc + token.text().len());

        if is_selectable_kind(parent.kind()) {
            self.selectable += 1;
        }

        self.tokens.push((parent, tokens));
    }

    pub fn render(&self, current_line: usize, cursor: (usize, usize)) -> Line<'_> {
        let mut spans = Vec::new();

        if let Some(ws) = &self.preceding_whitespace {
            spans.push(Span::from(ws));
        }

        let mut selectable = 0;

        for (parent, tokens) in &self.tokens {
            let parent_kind = parent.kind();

            let s = tokens.iter().fold(String::new(), |mut acc, token| {
                acc += token.text();
                acc
            });

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
                    if cursor.1 >= self.selectable && selectable + 1 == self.selectable {
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
