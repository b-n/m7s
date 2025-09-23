use log::debug;
use ratatui::{
    style::{Color, Stylize},
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
    ArrayItem(String),
    Kvp(String, String),
    Text(String),
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
    Regex::new(r#"^(?<key>\"[^\"]+"|'[^\']+'|[^\'\"]+):(?<value>.*)$"#)
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
        let content = if let Some(caps) = ARRAY_REGEX.captures(rest) {
            let value = caps.name("value").unwrap().as_str();
            LineContent::ArrayItem(value.to_string())
        } else if let Some(caps) = KEY_REGEX.captures(rest) {
            let key = caps.name("key").unwrap().as_str();
            let value = caps.name("value").unwrap().as_str();

            match (key, value) {
                ("", b) => LineContent::Text(b.to_string()),
                (a, b) => LineContent::Kvp(a.to_string(), b.to_string()),
            }
        } else {
            LineContent::Text(rest.into())
        };

        Ok(Self {
            indent: ws.to_string(),
            content,
        })
    }
}

impl FileLine {
    fn render(&self) -> Line<'_> {
        let mut output: Vec<Span<'_>> = vec![(&self.indent).into()];

        match &self.content {
            LineContent::ArrayItem(v) => {
                output.extend_from_slice(&["-".into(), " ".into(), v.into()]);
            }
            LineContent::Text(s) => output.extend_from_slice(&[s.into()]),
            LineContent::Kvp(k, v) => output.extend_from_slice(&[
                k.clone().bold().fg(Color::Yellow),
                ":".into(),
                v.into(),
            ]),
        }

        Line::from(output)
    }
}

#[derive(Debug)]
struct FileLines(Vec<FileLine>);

impl FileLines {
    fn render(&self) -> Vec<Line<'_>> {
        self.0.iter().map(|l| l.render()).collect()
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
}

impl File {
    pub fn from_path(path: PathBuf) -> Self {
        debug!("Loading file");
        //TODO: Make this falliable
        let contents = std::fs::read_to_string(&path).unwrap();
        let lines = contents.parse().unwrap();
        debug!("Parsed lines: {lines:#?}");

        Self { path, lines }
    }

    pub fn display_lines(&self, _cursor: usize) -> Vec<Line<'_>> {
        self.lines.render()
    }
}
