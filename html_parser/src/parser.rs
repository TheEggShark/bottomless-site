use super::scanner::{Token, TokenType, IDENTIFER_TOKENS};
use super::tag::Tag;

use std::collections::HashMap;
use std::fmt::Display;
use std::error::Error;

pub(crate) struct Parser {
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
            tags.push(tag);
        }

        Ok(tags)
    }

    fn tag(&mut self, source: &str) -> Result<Tag, ParseError> {
        self.consume(TokenType::LessThan)?;
        let next_token = self.advance();

        if !next_token.is_identifier() {
            Err(ParseError::UnexpectedToken {
                expected_tokens: IDENTIFER_TOKENS.to_vec(),
                incorect_token: next_token,
            })?;
        }
        let mut base_tag = Tag::from_token(next_token, source);

        // add atributes
        // should be ident or >
        loop {
            self.skip_white_space();
            let next_token = self.advance();
            match next_token.get_type() {
                TokenType::Identifier => {
                    let (name, value) = self.attribute(next_token, source)?;
                    base_tag.insert_attribute(name, value);
                }
                TokenType::ForwardSlash => {
                    // gotta love HTML optional / 
                    match base_tag {
                        Tag::NonCloseableTag { .. } => {
                            self.consume(TokenType::GreaterThan)?;
                            break;
                        },
                        Tag::CloseableTag { .. } => Err(ParseError::UnexpectedToken {
                            expected_tokens: vec![TokenType::Identifier, TokenType::GreaterThan],
                            incorect_token: next_token,
                        })?,
                    }
                },
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

        let mut content = String::new();
        loop {
            self.skip_all_text(&mut content, source);
            let non_text_token = self.peek();
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
                            tag_to_be_closed_char_pos: ident.get_character_pos(),
                            tag_to_be_closed_line_number: ident.get_line_number(),
                            tag_should_be_closed: ident_name.to_string(),
                        })?;
                    }
                    self.consume(TokenType::GreaterThan)?;
                    base_tag.add_content(&content);

                    break;
                },
                TokenType::LessThan => {
                    self.advance();
                    let next = self.peek();
                    if next.get_type() == TokenType::WhiteSpace {
                        content.push_str(non_text_token.get_str_representation(source));
                        content.push_str(next.get_str_representation(source));
                        continue;
                    }
                    //un-consumes the <
                    self.go_back(1);

                    let child = self.tag(source)?;
                    base_tag.add_child(child);
                },
                TokenType::Eof => {
                    Err(ParseError::UnterminatedTag {
                        unclosed_tag: base_tag.get_name().to_string(),
                        unclosed_line_number: base_tag.get_line_number(),
                        unclosed_char_pos: base_tag.get_character_pos(),
                    })?;
                }
                _ => {
                    println!("{:?}\n{:?}", base_tag, non_text_token);
                    unreachable!()
                }
            }
        }

        base_tag.clean_content();

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
                attributes: HashMap::new(),
                line_number: start_token.get_line_number(),
                start_char: start_token.get_character_pos(),
            }
        )
    }

    fn attribute(&mut self, ident: Token, source: &str) -> Result<(String, Option<String>), ParseError> {
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
            Ok((name, Some(value)))
        } else {
            Ok((name, None))
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
        if self.peek().is_identifier() {
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
            Bang | Identifier | Equal | String | WhiteSpace | Doctype |
            GreaterThan | Area | Base | Br | Col | Embed | Hr | Img | Input |
            Link | Meta | Param | Source | Track | Wbr | SomethingElse | ForwardSlash
        ) {
            let token = self.advance();
            let text_of_token = token.get_str_representation(source);
            buffer.push_str(text_of_token);
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken{
        expected_tokens: Vec<TokenType>,
        incorect_token: Token,
    },
    IncorrectTermination {
        tag_to_be_closed: String,
        tag_to_be_closed_line_number: usize,
        tag_to_be_closed_char_pos: usize,
        tag_should_be_closed: String,
    },
    UnterminatedTag {
        unclosed_tag: String,
        unclosed_line_number: usize,
        unclosed_char_pos: usize,
    },
    IncorrectDoctype,
}

impl Error for ParseError {
    
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken { expected_tokens, incorect_token } => {
                let mut string = String::from("expected: ");

                for token_type in expected_tokens {
                    let s = format!("{} ", token_type);
                    string.push_str(&s);
                }

                string.push_str("found: ");
                let ic_text = format!("{}, at line: {} char: {}", incorect_token.get_type(), incorect_token.get_line_number(), incorect_token.get_character_pos());
                string.push_str(&ic_text);

                write!(f, "{}", string)
            }
            Self::IncorrectTermination {
                tag_to_be_closed,
                tag_to_be_closed_char_pos,
                tag_to_be_closed_line_number,
                tag_should_be_closed,
            } => {
                write!(f,
                    "{} tag improperly closed found {} tag at line: {}, char: {}",
                    tag_to_be_closed,
                    tag_should_be_closed,
                    tag_to_be_closed_line_number,
                    tag_to_be_closed_char_pos,
                )
            },
            Self::UnterminatedTag {
                unclosed_tag,
                unclosed_line_number,
                unclosed_char_pos,
            } => {
                write!(
                    f,
                    "Unterminated {} tag at line number: {} char pos: {}",
                    unclosed_tag,
                    unclosed_line_number,
                    unclosed_char_pos,
                )
            }
            Self::IncorrectDoctype => write!(f, "DOCTYPE tag did not contain HTML")
        }
    }
}