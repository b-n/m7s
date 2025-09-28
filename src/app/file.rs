use log::debug;
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};
use regex::Regex;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(thiserror::Error)]
pub enum ParseFileError {
    #[error("Failed to parse file lines")]
    Generic,
}

impl std::fmt::Debug for ParseFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")
    }
}

#[derive(Debug)]
pub enum LineContent {
    ArrayItem(Box<LineContent>),
    Kvp(String, String),
    Text(String),
}

impl FromStr for LineContent {
    type Err = ParseFileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let content = if let Some(caps) = ARRAY_REGEX.captures(s) {
            let value = caps
                .name("value")
                .unwrap()
                .as_str()
                .parse::<LineContent>()?;

            LineContent::ArrayItem(Box::new(value))
        } else if let Some(caps) = KEY_REGEX.captures(s) {
            let key = caps.name("key").unwrap().as_str();
            let value = if let Some(value) = caps.name("value") {
                value.as_str()
            } else {
                ""
            };

            match (key, value) {
                ("", b) => LineContent::Text(b.to_string()),
                (a, b) => LineContent::Kvp(a.to_string(), b.to_string()),
            }
        } else {
            LineContent::Text(s.into())
        };

        Ok(content)
    }
}

impl LineContent {
    fn render(&self, active_line: bool, first_element: bool) -> Vec<Span<'_>> {
        match &self {
            LineContent::ArrayItem(v) => {
                let mut output = vec!["-".into(), " ".into()];
                output.extend(v.as_ref().render(active_line, first_element));
                output
            }
            LineContent::Text(s) => {
                let mut span = Span::from(s);
                if active_line {
                    span = span.reversed();
                }
                vec![span]
            }
            LineContent::Kvp(k, v) => {
                let mut key = Span::styled(k, Style::default().bold().fg(Color::Yellow));
                let mut value = Span::from(v);
                if active_line {
                    if first_element {
                        key = key.reversed();
                    } else {
                        value = value.reversed();
                    }
                }
                vec![key, ": ".into(), value]
            }
        }
    }
}

#[derive(Debug)]
struct FileLine {
    indent: String,
    content: LineContent,
}

static WS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?<ws>\s*)(?<rest>\S+.*)$").expect("Should always compile"));

static ARRAY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-\ (?<value>.*)$").expect("Should always compile"));

static KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(?<key>\"[^\"]+"|'[^\']+'|[^\'\"]+):($|\ (?<value>.*))$"#)
        .expect("Should always compile")
});

impl FromStr for FileLine {
    type Err = ParseFileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // SAFETY: unwraps on regex capture groups are generally safe as they would not be
        // available if the regex did not match
        let (ws, rest) = match WS_REGEX.captures(s) {
            Some(caps) => (
                caps.name("ws").unwrap().as_str(),
                caps.name("rest").unwrap().as_str(),
            ),
            None => ("", s),
        };

        let content = rest.parse::<LineContent>()?;

        Ok(Self {
            indent: ws.to_string(),
            content,
        })
    }
}

impl FileLine {
    fn render(&self, active_line: bool, first_element: bool) -> Line<'_> {
        let mut output: Vec<Span<'_>> = vec![(&self.indent).into()];
        let contents: Vec<Span<'_>> = self.content.render(active_line, first_element);
        output.extend(contents);

        Line::from(output)
    }
}

#[derive(Debug)]
struct FileLines(Vec<FileLine>);

impl FileLines {
    fn render(&self, cursor: (usize, bool)) -> (Vec<Line<'_>>, usize) {
        let (cursor_line, first_element) = cursor;

        self.0.iter().enumerate().fold((vec![], 0), |acc, l| {
            let (mut out, m) = acc;
            let (index, line) = l;
            let l = line.render(index == cursor_line, first_element);
            let m = m.max(l.width());

            out.push(l);
            (out, m)
        })
    }

    fn max_width(&self) -> usize {
        self.0
            .iter()
            .fold(0, |acc, l| acc.max(l.render(false, false).width()))
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}

impl FromStr for FileLines {
    type Err = ParseFileError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[allow(clippy::redundant_closure_for_method_calls)]
        let v = s
            .lines()
            .map(|l| l.parse::<FileLine>())
            .collect::<Result<Vec<FileLine>, ParseFileError>>()?;
        Ok(FileLines(v))
    }
}

impl IntoIterator for FileLines {
    type Item = FileLine;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    lines: FileLines,
    pub max_width: usize,
    pub line_count: usize,
}

impl File {
    pub fn from_path(path: PathBuf) -> Self {
        debug!("Loading file");
        //TODO: Make this falliable
        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: FileLines = contents.parse().unwrap();
        debug!("Parsed lines: {lines:#?}");

        // TODO: Maybe a better way to handle this?
        let max_width = lines.max_width();
        let line_count = lines.count();

        Self {
            path,
            lines,
            max_width,
            line_count,
        }
    }

    pub fn display_lines(&self, cursor: (usize, bool)) -> (Vec<Line<'_>>, usize) {
        self.lines.render(cursor)
    }
}
