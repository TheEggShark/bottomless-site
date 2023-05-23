mod scanner;
use scanner::{Scanner, Token, TokenType, IDENTIFER_TOKENS};

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
            self.skip_white_space();
            //parse it up
            let tag = self.tag(&source)?;
            //println!("{:?}", tag);
            tags.push(tag);
        }

        Ok(tags)
    }

    fn tag(&mut self, source: &str) -> Result<Tag, ParseError> {
        self.consume(TokenType::LessThan)?;
        let next_token = self.advance();
        let next_token_type = next_token.get_type();
        if !matches!(
            next_token_type,
            TokenType::Identifier | TokenType::Head | TokenType::Link | TokenType::Meta |
            TokenType::Script | TokenType::Style | TokenType::Title
        ) {
            Err(ParseError::UnexpectedToken {
                expected_tokens: vec![
                    TokenType::Identifier, TokenType::Head, TokenType::Link, TokenType::Meta,
                    TokenType::Script, TokenType::Style, TokenType::Title
                ],
                incorect_token: next_token,
            })?;
        }
        let mut base_tag = Tag::from_token(next_token, source);
        println!("{:?}", base_tag);

        // add atributes
        // should be ident or >
        loop {
            self.skip_white_space();
            let next_token = self.advance();
            match next_token.get_type() {
                TokenType::Identifier => {
                    let attribute = self.attribute(next_token, source)?;
                    base_tag.add_attribute(attribute);
                }
                TokenType::GreaterThan => {
                    break;
                }
                _ => Err(ParseError::UnexpectedToken {
                    expected_tokens: vec![TokenType::Identifier, TokenType::GreaterThan],
                    incorect_token: next_token,
                })?,
            } 
        }

        if matches!(base_tag, Tag::NonCloseableTag {..}) {
            return Ok(base_tag)
        }

        println!("WEEWOOOWEEWOO");

        let mut content = String::new();
        'child: loop {
            self.skip_all_text(&mut content, source);
            let non_text_token = self.peek();
            // println!("Non_text_token: {:?}", non_text_token);
            match non_text_token.get_type() {
                TokenType::CloseTag => {
                    // goes past the </
                    self.advance();
                    let ident = self.consume_identifer_like_token()?;
                    let ident_name = ident.get_str_representation(source);
                    let base_tag_name = base_tag.get_name();
                    if ident_name != base_tag_name {
                        Err(ParseError::IncorrectTermination {
                            tag_to_be_closed: base_tag_name.to_string(),
                            tag_should_be_closed: ident_name.to_string(),
                        })?;
                    }
                    self.consume(TokenType::GreaterThan)?;
                    base_tag.add_content(&content);
                    // println!("{:?}", base_tag);
                    // println!("break terminated: {}", ident_name);
                    break 'child;
                },
                TokenType::LessThan => {
                    self.advance();
                    let next = self.peek();
                    if next.get_type() == TokenType::WhiteSpace {
                        content.push_str(non_text_token.get_str_representation(source));
                        content.push_str(next.get_str_representation(source));
                        continue 'child;
                    }
                    //un-consumes the <
                    self.go_back(1);

                    let child = self.tag(source)?;
                    base_tag.add_child(child);
                },
                TokenType::Eof => {
                    Err(ParseError::UnterminatedTag)?;
                }
                _ => {
                    unreachable!()
                }
            }
        }


        // content parsing
        // ignore everything untill < or </

        Ok(base_tag)
    }

    fn doctype(&mut self, source: &str) -> Result<Tag, ParseError> {
        let start_token = self.consume(TokenType::LessThan)?;
        self.consume(TokenType::Bang)?;
        self.consume(TokenType::Doctype)?;
        self.consume(TokenType::WhiteSpace)?;
        let ident = self.consume(TokenType::Identifier)?;
        if ident.get_str_representation(source) != "html" {
            Err(ParseError::IncorrectDoctype)?;
        }
        self.skip_white_space();
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

    fn attribute(&mut self, ident: Token, source: &str) -> Result<Attribute, ParseError> {
        self.skip_white_space();
        //could be '=' or nothing could advance?
        let name = ident.get_str_representation(source).to_string();
        let next_token = self.peek();
        if next_token.get_type() == TokenType::Equal {
            self.advance();
            self.skip_white_space();
            let value = self.consume(TokenType::String)?
                .get_str_representation(source)
                .to_string();
            Ok(Attribute::new(name, Some(value)))
        } else {
            Ok(Attribute::new(name, None))
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.get_previous()
    }

    fn go_back(&mut self, amount: usize) {
        match self.current.checked_sub(amount) {
            Some(value) => self.current = value,
            None => self.current = 0,
        }
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

    fn check_token_type(&self, token_type: TokenType) -> bool {
        !self.is_at_end() && self.peek().get_type() == token_type
    }

    fn consume(&mut self, expected: TokenType) -> Result<Token, ParseError> {
        if self.check_token_type(expected) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken{
                expected_tokens: vec![expected],
                incorect_token: self.peek(),
            })
        }
    }

    fn consume_identifer_like_token(&mut self) -> Result<Token, ParseError> {
        use TokenType::*;
        if matches!(
            self.peek().get_type(),
            Doctype | Head | Meta | Title | Style |
            Link | Script | Base | Identifier
        )  {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected_tokens: IDENTIFER_TOKENS.to_vec(),
                incorect_token: self.peek()
            })
        }
    }

    // used in the case of optional white space ofc HTML
    // does not ignore whitespace bc pain
    fn skip_white_space(&mut self) {
        if self.peek().get_type() == TokenType::WhiteSpace {
            self.advance();
        } 
    }

    fn skip_all_text(&mut self, buffer: &mut String, source: &str) {
        use TokenType::*;
        while matches!(
            self.peek().get_type(),
            Bang | Identifier | Equal | String | WhiteSpace | Doctype | GreaterThan |
            Head | Meta | Title | Style | Link | Script | Base | SomethingElse
        ) {
            let token = self.advance();
            let text_of_token = token.get_str_representation(source);
            buffer.push_str(text_of_token);
        }
    }
}

#[derive(Debug)]
enum ParseError {
    UnexpectedToken{
        expected_tokens: Vec<TokenType>,
        incorect_token: Token,
    },
    IncorrectTermination {
        tag_to_be_closed: String,
        tag_should_be_closed: String,
    },
    UnterminatedTag,
    IncorrectDoctype,
}

// maybe switch to &str but then life times
#[derive(Debug)]
struct Attribute {
    name: String,
    value: Option<String>,
}

impl Attribute {
    pub fn new(name: String, value: Option<String>) -> Self {
        Self {
            name,
            value,
        }
    }
}

#[derive(Debug)]
enum Tag {
    //Tag -> LessThan Ident Atrributes grater than
    // Atriibutes -> [atrribute]
    // like <link> and <meta>
    CloseableTag {
        name: String,
        attributes: Vec<Attribute>,
        content: String,
        children: Vec<Tag>,
        line_number: usize,
        start_char: usize,
    },
    NonCloseableTag {
        name: String,
        attributes: Vec<Attribute>,
        line_number: usize,
        start_char: usize,
    },
}

impl Tag {
    pub fn from_token(token: Token, source: &str) -> Self {
        use TokenType::*;
        let token_type = token.get_type();
        let name = token.get_str_representation(source).to_string();
        let line_number = token.get_line_number();
        let start_char = token.get_character_pos();
        println!("{}", name);
        match token_type {
            Head | Script | Style | Title => Tag::new_closeable_tag(name, line_number, start_char),
            Link | Meta => Tag::new_noncloseable_tag(name, line_number, start_char),
            Identifier => {
                if name == "html" {
                    Tag::new_noncloseable_tag(name, line_number, start_char)
                } else {
                    Tag::new_closeable_tag(name, line_number, start_char)
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn new_closeable_tag(name: String, line_number: usize, start_char: usize) -> Self {
        Self::CloseableTag {
            name,
            attributes: Vec::new(),
            content: String::new(),
            children: Vec::new(),
            line_number,
            start_char,
        }
    }

    pub fn new_noncloseable_tag(name: String, line_number: usize, start_char: usize) -> Self {
        Self::NonCloseableTag {
            name,
            attributes: Vec::new(),
            line_number,
            start_char,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Self::CloseableTag { name, ..} => name,
            Self::NonCloseableTag { name, ..} => name,
        }
    }

    pub(crate) fn add_content(&mut self, new_content: &str) {
        match self {
            Self::CloseableTag {content, ..} => content.push_str(new_content),
            Self::NonCloseableTag { .. } => panic!("Trying to add content to NonCloseableTag")
        }
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        match self {
            Self::NonCloseableTag {attributes, ..} => attributes.push(attribute),
            Self::CloseableTag {attributes, ..} => attributes.push(attribute),
        }
    }

    pub fn add_child(&mut self, child: Self) {
        match self {
            Self::NonCloseableTag{..} => panic!("Non Closeable Tags cannot have children"),
            Self::CloseableTag { children, ..} => children.push(child),
        }
    }
}