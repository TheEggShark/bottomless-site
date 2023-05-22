mod scanner;
use scanner::{Scanner, Token, TokenType};

pub fn parse_file(file: &str) {
    //start with < end with >
    let f = std::fs::read_to_string(file).unwrap();
    println!("len is {}", f.len());
    let mut scanner = Scanner::new(f);
    scanner.scan_tokens();

    let res = scanner.extract_source();
    let (tokens, source) = match res {
        Ok(stuff) => {stuff},
        Err(e) => {
            println!("{:?}", e);
            return;
        },
    };

    let mut parser = Parser::new(tokens);
    let tree = parser.parse(source).unwrap();
    println!("{:?}", tree);
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0
        }
    }

    pub fn parse(&mut self, source: String) -> Result<Vec<Tag>, ParseError> {
        let mut tags = Vec::new();

        let first_tag = self.doctype(&source)?;
        tags.push(first_tag);

        while !self.is_at_end() {
            //parse it up
            break;
        }

        Ok(tags)
    }

    fn tag(&mut self) -> Result<Tag, ParseError> {
        self.consume(TokenType::LessThan)?;
        let next_token = self.advance();
        if !matches!(
            next_token.get_type(),
            TokenType::Identifier | TokenType::Head | TokenType::Link | TokenType::Meta |
            TokenType::Script | TokenType::Style | TokenType::Title
        ) {
            Err(ParseError::UnexpectedToken {
                expected_token: TokenType::Identifier,
                incorect_token: next_token,
            })?;
        }

        todo!()
    }

    fn doctype(&mut self, source: &str) -> Result<Tag, ParseError> {
        let start_token = self.consume(TokenType::LessThan)?;
        self.consume(TokenType::Bang)?;
        self.consume(TokenType::Doctype)?;
        let ident = self.consume(TokenType::Identifier)?;
        if ident.get_str_representation(source) != "html" {
            Err(ParseError::IncorrectDoctype)?;
        }
        self.consume(TokenType::GreaterThan)?;
        Ok(
            Tag::NonCloseableTag { 
                name: "DOCTYPE".to_string(),
                attributes: Vec::new(),
                line_number: start_token.get_line_number(),
                start_char: start_token.get_character_pos(),
            }
        )
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.get_previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current]
    }

    fn get_previous(&self) -> Token {
        self.tokens[self.current-1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().get_type() == TokenType::Eof
    }

    fn matches(&mut self, valid_types: &[TokenType]) -> bool {
        for token_type in valid_types {
            if self.check_token_type(*token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check_token_type(&self, token_type: TokenType) -> bool {
        !self.is_at_end() && self.peek().get_type() == token_type
    }

    fn consume(&mut self, expected: TokenType) -> Result<Token, ParseError> {
        if self.check_token_type(expected) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken{
                expected_token: expected,
                incorect_token: self.peek(),
            })
        }
    }
}

#[derive(Debug)]
enum ParseError {
    UnexpectedToken{
        expected_token: TokenType,
        incorect_token: Token,
    },
    UnterminatedTag,
    IncorrectDoctype,
}

// maybe switch to &str but then life times
#[derive(Debug)]
struct Atribute {
    name: String,
    value: String,
}

#[derive(Debug)]
enum Tag {
    //Tag -> LessThan Ident Atrributes grater than
    // Atriibutes -> [atrribute]
    // like <link> and <meta>
    CloseableTag {
        name: String,
        atrributtes: Vec<Atribute>,
        content: String,
        children: Vec<Tag>,
        line_number: usize,
        start_char: usize,
    },
    NonCloseableTag {
        name: String,
        attributes: Vec<Atribute>,
        line_number: usize,
        start_char: usize,
    },
}

enum TagType {
    Doctype,
    Head,
    Meta,
    Title,
    Style,
    Link,
    Script,
    Base,
    Unknown {
        name: String,
    }
}

impl ToString for TagType {
    fn to_string(&self) -> String {
        use TagType::*;
        match self {
            Doctype => String::from("DOCTYPE"),
            Head => String::from("head"),
            Meta => String::from("meta"),
            Title => String::from("title"),
            Style => String::from("style"),
            Link => String::from("link"),
            Script => String::from("script"),
            Base => String::from("base"),
            Unknown { name } => String::from(name),
        }
    }
}