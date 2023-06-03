use std::collections::HashMap;

use super::scanner::{Token, TokenType};

#[derive(Debug)]
pub enum Tag {
    //Tag -> LessThan Ident Atrributes grater than
    // Atriibutes -> [atrribute]
    // like <link> and <meta>
    CloseableTag {
        name: String,
        attributes: HashMap<String, Option<String>>,
        content: String,
        children: Vec<Tag>,
        line_number: usize,
        start_char: usize,
    },
    NonCloseableTag {
        name: String,
        attributes: HashMap<String, Option<String>>,
        line_number: usize,
        start_char: usize,
    },
}

#[derive(Debug)]
pub struct IterTag {
    name: String,
    attributes: HashMap<String, Option<String>>,
    content: Option<String>,
    line_number: usize,
    start_char: usize,
}

impl IterTag {
    pub fn new(name: String, attributes: HashMap<String, Option<String>>, content: Option<String>, ln: usize, sc: usize) -> Self {
        Self {
            name,
            attributes,
            content,
            line_number: ln,
            start_char: sc,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_content(&self) -> &Option<String> {
        &self.content
    }

    pub fn get_line_number(&self) -> usize {
        self.line_number
    }

    pub fn get_start_char(&self) -> usize {
        self.start_char
    }

    pub fn get_attribute(&self, name: &str) -> &Option<String> {
        self.attributes.get(name).unwrap_or(&None)
    }
}

impl Tag {
    pub fn from_token(token: Token, source: &str) -> Self {
        use TokenType::*;
        let token_type = token.get_type();
        let name = token.get_str_representation(source).to_string();
        let line_number = token.get_line_number();
        let start_char = token.get_character_pos();

        match token_type {
            Area | Base | Br | Col | Embed | Hr | Img | Input |
            Link | Meta | Param | Source | Track | Wbr => Tag::new_noncloseable_tag(name, line_number, start_char),
            Identifier => Tag::new_closeable_tag(name, line_number, start_char),
            _ => unreachable!(),
        }
    }

    pub fn new_closeable_tag(name: String, line_number: usize, start_char: usize) -> Self {
        Self::CloseableTag {
            name,
            attributes: HashMap::new(),
            content: String::new(),
            children: Vec::new(),
            line_number,
            start_char,
        }
    }

    pub fn new_noncloseable_tag(name: String, line_number: usize, start_char: usize) -> Self {
        Self::NonCloseableTag {
            name,
            attributes: HashMap::new(),
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

    pub fn get_attributes(&self) -> &HashMap<String, Option<String>> {
        match self {
            Self::CloseableTag { attributes, .. } => attributes,
            Self::NonCloseableTag { attributes, .. } => attributes,
        }
    }

    pub fn get_line_number(&self) -> usize {
        match self {
            Self::CloseableTag { line_number, ..} => *line_number,
            Self::NonCloseableTag { line_number, ..} => *line_number,
        }
    }

    pub fn get_character_pos(&self) -> usize {
        match self {
            Self::CloseableTag { start_char, ..} => *start_char,
            Self::NonCloseableTag { start_char, ..} => *start_char,
        }
    }

    pub fn get_children(&self) -> Option<&[Tag]> {
        match self {
            Self::NonCloseableTag{..} => None,
            Self::CloseableTag { children, ..} => Some(children),
        }
    }

    pub(crate) fn add_content(&mut self, new_content: &str) {
        match self {
            Self::CloseableTag {content, ..} => content.push_str(new_content),
            Self::NonCloseableTag { .. } => panic!("Trying to add content to NonCloseableTag")
        }
    }

    pub fn insert_attribute(&mut self, name: String, value: Option<String>) {
        match self {
            Self::NonCloseableTag {attributes, ..} => attributes.insert(name, value),
            Self::CloseableTag {attributes, ..} => attributes.insert(name, value),
        };
    }

    pub fn add_child(&mut self, child: Self) {
        match self {
            Self::NonCloseableTag{..} => panic!("Non Closeable Tags cannot have children"),
            Self::CloseableTag { children, ..} => children.push(child),
        }
    }

    pub fn format_tag(&self, depth: usize) -> String {
        let mut text = String::new();

        add_tabs(depth, &mut text);
        text.push_str("{\n");

        add_tabs(depth+1, &mut text);
        let name = format!("Name: {}\n", self.get_name());
        text.push_str(&name);
        add_tabs(depth+1, &mut text);

        text.push_str("Attributes: [\n");
        for (name, value) in self.get_attributes() {
            add_tabs(depth+2, &mut text);
            text.push_str(name);
            match value {
                Some(val) => {
                    let attribute_text = format!("={},", val);
                    text.push_str(&attribute_text);
                },
                None => {},
            }
            text.push('\n');
        }

        add_tabs(depth+1, &mut text);
        text.push_str("]\n");

        if let Some(content) = self.get_content() {
            add_tabs(depth+1, &mut text);
            let content_text = format!("Content: {}\n", content);
            text.push_str(&content_text);
        }

        if let Some(children) = self.get_children() {
            add_tabs(depth+1, &mut text);
            text.push_str("Chilren: [\n");
            for child in children {
                let child = child.format_tag(depth+2);
                text.push_str(&child);
                text.push_str(",\n");
            }
            add_tabs(depth+1, &mut text);
            text.push_str("]\n");
        }

        add_tabs(depth, &mut text);
        text.push('}');

        text
    }

    pub fn get_content(&self) -> Option<&str> {
        match self {
            Self::CloseableTag { content, ..} => Some(content),
            Self::NonCloseableTag { .. } => None,
        }
    }


    pub fn clean_content(&mut self) {
        match self {
            Self::CloseableTag { content, ..} => {
                if is_all_white_space(content) {
                    *content = String::new();
                } else {
                    // html does not are if you have a new line
                    // so this removes the new line and keeps the single space between words
                    // although if there is say "hello  my name" this does remove the double spacing
                    *content = content.split("\n")
                    .flat_map(|s| {
                        [s.trim_start(), " "]
                    }).collect();
                    *content = content.trim_end().to_string();
                }
            },
            Self::NonCloseableTag { .. } => {}
        }
    }
}

fn add_tabs(num_of_tabs: usize, text: &mut String) {
    for _ in 0..num_of_tabs {
        text.push_str("   ");
    }
}

fn is_all_white_space(content: &str) -> bool {
    content.chars().all(char::is_whitespace)
}