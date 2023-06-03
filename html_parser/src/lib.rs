pub mod scanner;
pub mod parser;
pub mod tag;

use std::error::Error;
use std::fmt::Display;

use scanner::Scanner;
use parser::Parser;
use tag::{Tag, IterTag};

pub fn parse_file(file: &str) -> Result<Vec<Tag>, HTMLError> {
    //start with < end with >
    let f = std::fs::read_to_string(file)?;

    let mut scanner = Scanner::new(f);
    scanner.scan_tokens();

    let (tokens, source) = scanner.extract_source()?;

    let mut parser = Parser::new(tokens);
    let tree = parser.parse(source)?;

    Ok(tree)
}

#[derive(Debug)]
pub enum HTMLError {
    LexicalError(scanner::LexicalError),
    ParseError(parser::ParseError),
    IoError(std::io::Error),
}

impl Display for HTMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LexicalError(e) => write!(f, "{}", e),
            Self::ParseError(e) => write!(f, "{}", e),
            Self::IoError(e) => write!(f, "{}", e),
        }
    }
}

impl From<scanner::LexicalError> for HTMLError {
    fn from(value: scanner::LexicalError) -> Self {
        Self::LexicalError(value)
    }
}

impl From<parser::ParseError> for HTMLError {
    fn from(value: parser::ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl From<std::io::Error> for HTMLError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl Error for HTMLError {

}

pub fn flaten_tree(tree: Vec<Tag>) -> Vec<IterTag> {
    tree.into_iter()
        .flat_map(tag_to_iter_tag)
        .collect()
}

pub fn tag_to_iter_tag(tag: Tag) -> Vec<IterTag> {
    match tag {
        Tag::NonCloseableTag {
            name,
            attributes,
            line_number,
            start_char,
        } => vec![IterTag::new(name, attributes, None, line_number, start_char)],
        Tag::CloseableTag {
            name,
            attributes,
            content,
            children,
            line_number,
            start_char,
        } => {
            let mut tags = Vec::new();

            tags.push(IterTag::new(name, attributes, Some(content), line_number, start_char));

            for child in children {
                let iter_tag = tag_to_iter_tag(child);
                tags.extend(iter_tag.into_iter());
            }
            tags
        }
    }
}