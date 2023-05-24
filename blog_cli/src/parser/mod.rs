mod scanner;
mod parser;
mod tag;

use scanner::Scanner;
use parser::Parser;

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
    for tag in tree {
        println!("{}", tag.format_tag(0));
    }
}