use log::debug;
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(thiserror::Error)]
pub enum ParseFileError {
    #[error("Capture group missing")]
    MissingCaptureGroup,
}

impl std::fmt::Debug for ParseFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")
    }
}

#[derive(Debug)]
struct FileLine {
    indent: String,
    length: usize,
}

impl FileLine {
    fn render(&self, _cursor: (usize, usize)) -> Line<'_> {
        todo!()
    }
}

/// Wrapper struc to implment FromStr which will use a yaml parser
#[derive(Debug)]
struct FileLines(Vec<FileLine>);

impl FromStr for FileLines {
    type Err = ParseFileError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

//impl IntoIterator for FileLines {
//    type Item = FileLine;
//    type IntoIter = std::vec::IntoIter<Self::Item>;
//
//    fn into_iter(self) -> Self::IntoIter {
//        self.0.into_iter()
//    }
//}

impl FileLines {
    fn iter(&self) -> std::slice::Iter<'_, FileLine> {
        self.0.iter()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

// TODO: Save file
#[derive(Debug)]
pub struct File {
    path: PathBuf,
    pub max_width: usize,
    pub line_count: usize,
    raw: String,
    lines: FileLines,
}

impl File {
    pub fn from_path(path: PathBuf) -> Self {
        debug!("Loading file");
        //TODO: Make this falliable
        let raw = std::fs::read_to_string(&path).unwrap();
        let lines: FileLines = raw.parse().unwrap();
        debug!("Parsed lines: {lines:#?}");

        // TODO: Maybe a better way to handle this?
        //let max_width = lines.max_width();
        let max_width = 100;
        let line_count = lines.len();

        Self {
            path,
            lines,
            max_width,
            line_count,
            raw,
        }
    }

    pub fn render(&self, cursor: (usize, usize)) -> (Vec<Line<'_>>, usize) {
        let lines = self.lines.iter().map(|line| line.render(cursor)).collect();

        (lines, self.max_width)
    }

    pub fn info(&self, _cursor: (usize, usize)) {
        use yaml_parser::ast::AstNode;
        let tree = yaml_parser::parse(&self.raw).unwrap();

        let root_ast = <yaml_parser::ast::Root>::cast(tree).unwrap();
        log::debug!("AST: {root_ast:#?}");

        let docs = root_ast.documents();
        log::debug!("Docs: {docs:#?}");

        for doc in docs {
            log::debug!("Doc: {doc:#?}");
            log::debug!("Doc: {:?}", doc.syntax());
        }
    }
}
