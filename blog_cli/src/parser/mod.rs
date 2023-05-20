mod scanner;
use scanner::Scanner;

pub fn parse_file(file: &str) {
    //start with < end with >
    let f = std::fs::read_to_string(file).unwrap();
    println!("len is {}", f.len());
    let mut scanner = Scanner::new(f);
    scanner.scan_tokens();

    let res = scanner.extract_source();
    match res {
        Ok(_) => {},
        Err(e) => println!("{:?}", e),
    }
}

//Grammar is always tag: name atributes Optional-ish<body> Optional-ish<closing>
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


trait Tag {

}