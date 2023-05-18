use std::{ops::Range, collections::HashMap};

pub fn parse_file(file: &str) {
    //start with < end with >
    let f = std::fs::read_to_string(file).unwrap();
    println!("len is {}", f.len());
    let mut scanner = Scanner::new(f);
    scanner.scan_tokens();
}

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line_number: usize,
    reserved_words: HashMap<String, TokenType>,
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
            reserved_words,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        };

        self.print_lexemes();
    }

    pub fn scan_token(&mut self) {
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
            "\n" => self.line_number += 1,
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
        while self.peek() != Some("\"") && !self.is_at_end() {
            if self.peek() == Some("\n") {
                self.line_number += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            panic!("Unterminated string at {}", self.line_number);
        }

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
        while self.peek_n(3) != Some("-->") {
            if self.peek() == Some("\n") {
                self.line_number += 1;
            }
            self.advance();
        }

        self.advance();
        self.advance();
        self.advance();
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
        self.tokens.push(Token::new(self.start..self.current, token_type, self.line_number));
    }

    fn print_lexemes(&self) {
        for token in self.tokens.iter() {
            token.display();
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

    map.insert("head".to_string(), TokenType::Head);
    map.insert("meta".to_string(), TokenType::Meta);
    
    map
}

fn is_alphanumeric(input: &str) -> bool {
    // want to include - for certian atributes
    input.chars().all(|c| char::is_alphanumeric(c) || c == '-')
}

fn is_aplha(input: &str) -> bool {
    input.chars().all(|c| char::is_alphabetic(c) || c == '-')
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    lexeme_start: usize,
    lexeme_end: usize,
    line_number: usize,
}

impl Token {
    pub fn new(lexeme_location: Range<usize>, token_type: TokenType, line_number: usize) -> Self {
        Self {
            token_type,
            lexeme_start: lexeme_location.start,
            lexeme_end: lexeme_location.end,
            line_number,
        }
    }

    pub fn get_str_representation<'a>(&'a self, source: &'a str) -> &str {
        &source[self.lexeme_start..self.lexeme_end]
    }

    pub fn display(&self) {
        print!("{:?}: line_number: {} ", self.token_type, self.line_number);
    }

    pub fn get_type(&self) -> TokenType {
        self.token_type
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TokenType {
    // regular sytax
    Bang,
    LessThan,
    GreaterThan,
    CloseTag,
    Identifier,
    Equal,
    String,
    // reserved words
    Head,
    Meta,
}