use super::scanner::{Token, TokenType};

#[derive(Debug)]
pub enum Tag {
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

    pub fn get_attributes(&self) -> &[Attribute] {
        match self {
            Self::CloseableTag { attributes, .. } => attributes,
            Self::NonCloseableTag { attributes, .. } => attributes,
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

    pub fn format_tag(&self, depth: usize) -> String {
        let mut text = String::new();

        add_tabs(depth, &mut text);
        text.push_str("{\n");

        add_tabs(depth+1, &mut text);
        let name = format!("Name: {}\n", self.get_name());
        text.push_str(&name);
        add_tabs(depth+1, &mut text);

        text.push_str("Attributes: [\n");
        for attribute in self.get_attributes() {
            add_tabs(depth+2, &mut text);
            text.push_str(&attribute.name);
            match &attribute.value {
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

    pub fn print_content(&self) {
        match self {
            Self::CloseableTag { content, ..} => println!("{}", content),
            Self::NonCloseableTag { .. } => {}
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

#[derive(Debug)]
pub struct Attribute {
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

fn is_all_white_space(content: &str) -> bool {
    content.chars().all(char::is_whitespace)
}