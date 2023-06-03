use std::{ops::Range, collections::HashMap};
use std::fmt::Display;
use std::error::Error;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line_number: usize,
    chars_at_end_of_last_line: usize,
    reserved_words: HashMap<String, TokenType>,
    error: Option<LexicalError>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        let reserved_words = reserved_wrods();

        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line_number: 1,
            chars_at_end_of_last_line: 1,
            reserved_words,
            error: None,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        };

        self.add_eof();
    }

    pub fn extract_source(self) -> Result<(Vec<Token>, String), LexicalError> {
        match self.error {
            Some(e) => Err(e),
            None => Ok((self.tokens, self.source))
        }
    }

    fn scan_token(&mut self) {
        let char_at_current = self.advance();

        match char_at_current {
            "<" => {
                if self.peek_n(3) == Some("!--") {
                    self.comment();
                } else if self.peek() == Some("/") {
                    self.advance();
                    self.add_token(TokenType::CloseTag);
                } else {
                    self.add_token(TokenType::LessThan);
                }
            },
            ">" => self.add_token(TokenType::GreaterThan),
            "=" => self.add_token(TokenType::Equal),
            "!" => self.add_token(TokenType::Bang),
            "/" => self.add_token(TokenType::ForwardSlash),
            "\"" => self.string(),
            " " | "\r" | "\t" | "\n" => self.whitespace(),
            _ => {
                if is_alpha(char_at_current) {
                    self.identifier();
                }
                else {
                    // Like in a <p> there could be a '.' or a digit
                    // and like its not invalid but just adding special rules would
                    // be very annyoing? does this make everything much slower, 
                    // probably
                    self.add_token(TokenType::SomethingElse);
                }
            }
        }
    }

    fn string(&mut self) {
        let mut lines_to_add = 0;
        while self.peek() != Some("\"") && !self.is_at_end() {
            if self.peek() == Some("\n") {
                lines_to_add += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error = Some(LexicalError::UnterminatedString(self.line_number));
            return;
        }

        self.advance(); // consumes the closing "
        self.add_token(TokenType::String);
        self.new_line(lines_to_add);
    }

    fn identifier(&mut self) {
        while self.peek().is_some() && is_alphanumeric(self.peek().unwrap()) {
            self.advance();
        }

        let raw_text = &self.source[self.start..self.current];
        match self.reserved_words.get(raw_text) {
            Some(token) => self.add_token(*token),
            None => self.add_token(TokenType::Identifier),
        }
    }

    fn comment(&mut self) {
        let mut lines_to_add = 0;
        while self.peek_n(3) != Some("-->") && !self.is_at_end() {
            if self.peek() == Some("\n") {
                lines_to_add += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error = Some(LexicalError::UnterminatedComment(self.line_number));
            return;
        }

        self.new_line(lines_to_add);

        self.advance();
        self.advance();
        self.advance();
    }

    fn whitespace(&mut self) {
        let mut lines_to_add = 0;

        if self.peek_previous() == Some("\n") {
            lines_to_add += 1;
        }

        while self.peek().is_some() && is_white_space(self.peek().unwrap()) {
            if self.peek() == Some("\n") {
                lines_to_add += 1;
            }
            self.advance();
        }

        self.add_token(TokenType::WhiteSpace);
        // seems like backwards order but bc of its multiline nature this is nessicary
        self.new_line(lines_to_add);
    }

    fn new_line(&mut self, number_of_lines: usize) {
        if number_of_lines > 0 {
            self.line_number += number_of_lines;
            self.chars_at_end_of_last_line = self.current;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let char_pos = self.start+1 - self.chars_at_end_of_last_line;

        if token_type == TokenType::String {
            // takes out the "" from the string
            let token = Token::new(self.start+1..self.current-1, token_type, self.line_number, char_pos);
            self.tokens.push(token);
        } else {
            let token = Token::new(self.start..self.current, token_type, self.line_number, char_pos);
            self.tokens.push(token);
        }
    }

    fn add_eof(&mut self) {
        self.tokens.push(Token::new(0..0, TokenType::Eof, self.line_number, 0));
    }

    fn advance(&mut self) -> &str {
        // this was ignoreing the first character so I had to add this in
        let out = &self.source[self.current..self.current+1];
        self.current += 1;
        out
    }

    fn peek(&self) -> Option<&str> {
        if self.is_at_end() {
            return None;
        }

        Some(&self.source[self.current..self.current+1])
    }

    fn peek_previous(&self) -> Option<&str> {
        match self.current.checked_sub(1) {
            Some(value) => Some(&self.source[value..value+1]),
            None => None
        }
    }

    fn peek_n(&self, number: usize) -> Option<&str> {
        if number + self.current >= self.source.len() {
            return None;
        }

        Some(&self.source[self.current..self.current+number])
    }
}

fn reserved_wrods() -> HashMap<String, TokenType> {
    let mut map = HashMap::new();

    // TokenType::Area, TokenType::Base, TokenType::Br, TokenType::Col, TokenType::Embed,
    // TokenType::Hr, TokenType::Img, TokenType::Input, TokenType::Link, TokenType::Meta,
    // TokenType::Param, TokenType::Source, TokenType::Track, TokenType::Wbr, TokenType::Identifier

    let reserved_words = [
        ("area".to_string(), TokenType::Area),
        ("base".to_string(), TokenType::Base),
        ("br".to_string(), TokenType::Br),
        ("col".to_string(), TokenType::Col),
        ("embed".to_string(), TokenType::Embed),
        ("hr".to_string(), TokenType::Hr),
        ("img".to_string(), TokenType::Img),
        ("input".to_string(), TokenType::Input),
        ("link".to_string(), TokenType::Link),
        ("meta".to_string(), TokenType::Meta),
        ("param".to_string(), TokenType::Param),
        ("source".to_string(), TokenType::Source),
        ("track".to_string(), TokenType::Track),
        ("wbr".to_string(), TokenType::Wbr),
        ("DOCTYPE".to_string(), TokenType::Doctype),
    ];

    reserved_words.into_iter()
        .for_each(|(k,v)| { map.insert(k, v); });
    
    map
}

pub fn is_white_space(input: &str) -> bool {
    matches!(input, " " | "\r" | "\t" | "\n")
}

fn is_alphanumeric(input: &str) -> bool {
    // want to include - for certian atributes
    input.chars().all(|c| char::is_alphanumeric(c) || c == '-')
}

fn is_alpha(input: &str) -> bool {
    input.chars().all(|c| char::is_alphabetic(c) || c == '-')
}

#[derive(Clone, Copy)]
pub struct Token {
    token_type: TokenType,
    lexeme_start: usize,
    lexeme_end: usize,
    line_number: usize,
    character_pos: usize,
}

impl Token {
    fn new(lexeme_location: Range<usize>, token_type: TokenType, line_number: usize, character_pos: usize) -> Self {
        Self {
            token_type,
            lexeme_start: lexeme_location.start,
            lexeme_end: lexeme_location.end,
            line_number,
            character_pos
        }
    }

    pub fn get_line_number(&self) -> usize {
        self.line_number
    }

    pub fn get_character_pos(&self) -> usize {
        self.character_pos
    }

    pub fn get_str_representation<'a>(&'a self, source: &'a str) -> &str {
        &source[self.lexeme_start..self.lexeme_end]
    }

    pub fn get_type(&self) -> TokenType {
        self.token_type
    }

    pub fn is_identifier(&self) -> bool {
        use TokenType::*;
        matches!(
            self.token_type,
            Area | Base | Br | Col | Embed | Hr | Img | Input |
            Link | Meta | Param | Source | Track | Wbr | Identifier
        )
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("token")
            .field("token_type", &self.token_type)
            .field("line_number", &self.line_number)
            .field("character_pos", &self.character_pos)
            .finish()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType {
    // regular sytax
    Bang,
    LessThan,
    GreaterThan,
    CloseTag,
    Identifier,
    Equal,
    String,
    ForwardSlash,
    WhiteSpace, // turns out HTML does not ignore whitespace
    // take the case of <p> lorem < </p> in this case < is written as text
    // but <p> lorem <sds </p> breaks things and ofc there are expections
    // reserved words mostlity consists of all self closing tags used later 
    // for parser
    Doctype,
    Area,
    Base,
    Br,
    Col,
    Embed,
    Hr,
    Img,
    Input,
    Link,
    Meta,
    Param,
    Source,
    Track,
    Wbr,
    // cope tokens
    SomethingElse,
    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bang => write!(f, "!"),
            Self::LessThan => write!(f, "<"),
            Self::GreaterThan => write!(f, ">"),
            Self::CloseTag => write!(f, "/>"),
            Self::Identifier => write!(f, "Identifier"),
            Self::Equal => write!(f, "="),
            Self::String => write!(f, "String"),
            Self::ForwardSlash => write!(f, "/"),
            Self::WhiteSpace => write!(f, "WhiteSpace"),
            Self::Doctype => write!(f, "Doctype"),
            Self::Area => write!(f, "area"),
            Self::Base => write!(f, "base"),
            Self::Br => write!(f, "br"),
            Self::Col => write!(f, "col"),
            Self::Embed => write!(f, "embed"),
            Self::Hr => write!(f, "hr"),
            Self::Img => write!(f, "img"),
            Self::Input => write!(f, "input"),
            Self::Link => write!(f, "link"),
            Self::Meta => write!(f, "meta"),
            Self::Param => write!(f, "param"),
            Self::Source => write!(f, "source"),
            Self::Track => write!(f, "track"),
            Self::Wbr => write!(f, "wbr"),
            Self::SomethingElse => write!(f, "text"),
            Self::Eof => write!(f, "End of File"),
        }
    }
}

pub(crate) const IDENTIFER_TOKENS: [TokenType; 15] = [
    TokenType::Area, TokenType::Base, TokenType::Br, TokenType::Col, TokenType::Embed,
    TokenType::Hr, TokenType::Img, TokenType::Input, TokenType::Link, TokenType::Meta,
    TokenType::Param, TokenType::Source, TokenType::Track, TokenType::Wbr, TokenType::Identifier
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LexicalError {
    UnterminatedString(usize),
    UnterminatedComment(usize),
}

impl Display for LexicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnterminatedComment(num) => write!(f, "Unterminated comment at line: {}", num),
            Self::UnterminatedString(num) => write!(f, "Uniterminated comment at line: {}", num),
        }
    }
}

impl Error for LexicalError {
    
}