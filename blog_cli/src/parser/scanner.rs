use std::{ops::Range, collections::HashMap};

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

        self.print_lexemes();
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
            "\"" => self.string(),
            "\n" => self.new_line(1),
            " " | "\r" | "\t" => {}
            _ => {
                if is_aplha(char_at_current) {
                    self.identifier();
                } else {
                    self.advance();
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

        self.new_line(lines_to_add);

        self.advance(); // consumes the closing "
        self.add_token(TokenType::String);
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

    fn new_line(&mut self, number_of_lines: usize) {
        if number_of_lines > 0 {
            self.line_number += number_of_lines;
            self.chars_at_end_of_last_line = self.current;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() - 1 || self.end_of_head()
    }

    fn end_of_head(&self) -> bool {
        if self.tokens.len() < 2{
            return false;
        }

        let token_len = self.tokens.len();
        self.tokens[token_len-2].get_type() == TokenType::CloseTag && self.tokens[token_len-1].get_type() == TokenType::Head
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

    fn print_lexemes(&self) {
        for token in self.tokens.iter() {
            print!("{:?}", token);
            println!("{}", token.get_str_representation(&self.source));
        }
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

    fn peek_n(&self, number: usize) -> Option<&str> {
        if number + self.current >= self.source.len() {
            return None;
        }

        Some(&self.source[self.current..self.current+number])
    }
}

fn reserved_wrods() -> HashMap<String, TokenType> {
    let mut map = HashMap::new();

    let reserved_words = [
        ("head".to_string(), TokenType::Head),
        ("meta".to_string(), TokenType::Meta),
        ("title".to_string(), TokenType::Title),
        ("link".to_string(), TokenType::Link),
        ("script".to_string(), TokenType::Script),
        ("style".to_string(), TokenType::Style),
        ("base".to_string(), TokenType::Base),
        ("DOCTYPE".to_string(), TokenType::Doctype),
    ];

    reserved_words.into_iter()
        .for_each(|(k,v)| { map.insert(k, v); });
    
    map
}

fn is_alphanumeric(input: &str) -> bool {
    // want to include - for certian atributes
    input.chars().all(|c| char::is_alphanumeric(c) || c == '-')
}

fn is_aplha(input: &str) -> bool {
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
    // reserved words
    Doctype,
    Head,
    Meta,
    Title,
    Style,
    Link,
    Script,
    Base,
    // cope token
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LexicalError {
    UnterminatedString(usize),
    UnterminatedComment(usize),
}