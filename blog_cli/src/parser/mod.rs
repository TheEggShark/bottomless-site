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
    parser.parse().unwrap();
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

    pub fn parse(&mut self) -> Result<Vec<Tag>, ParseError> {
        //start with doctype
        let current_token = self.advance();
        let mut tags = Vec::new();

        while !self.is_at_end() {
            //parse it up
        }

        Ok(tags)
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
}

#[derive(Debug)]
enum ParseError {
    UnexpectedToken,
}

//Tag -> LessThan Ident Atrributes grater than
// Atriibutes -> [atrribute]
// like <link> and <meta>
struct NonCloseableTag {
    name: String,
    attributes: Vec<Atribute>,
}

// its just <script></script> and <title></title>
struct CloseableTag {
    name: String,
    atrributtes: Vec<Atribute>,
    content: String 
    // pretty sure all the tags within the head can contain more children
    // so we are going with this but we'll see.
}

// maybe switch to &str but then life times
struct Atribute {
    name: String,
    value: String,
}

enum Tag {
    CloseableTag {
        name: String,
        atrributtes: Vec<Atribute>,
        content: String,
        children: Vec<Tag>,
    },
    NonCloseableTag {
        name: String,
        attributes: Vec<Atribute>,
    },
}