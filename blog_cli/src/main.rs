use std::time::UNIX_EPOCH;

fn main() {
    println!("Welcome to Charlies Blog Metadata(.CBMD) creater");
    println!("Give the title of the blog");
    let mut title = String::new();
    std::io::stdin().read_line(&mut title).unwrap();
    println!("give me the path of the file!");
    let mut file_path = String::new();
    std::io::stdin().read_line(&mut file_path).unwrap();
    println!("Give the intro words!");
    let mut intro = String::new();
    std::io::stdin().read_line(&mut intro).unwrap();
    println!("{title}\n{file_path}\n{intro}");
}

fn file_path_to_seconds_since_epoch(path: &str) -> Result<u64, std::io::Error> {
    let path = std::path::Path::new(path);
    let medata_data = path.metadata()?.modified()?;
    let time_since_epoch = medata_data.duration_since(UNIX_EPOCH)
        .expect("file before UNIX EPOCH")
        .as_secs();

    Ok(time_since_epoch)
}