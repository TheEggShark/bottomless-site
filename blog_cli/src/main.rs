use blog_cli::Cbmd;

use std::{fs, ffi::OsStr};


fn main() {
    println!("generating CBMD!");

    let blog_folder = fs::read_dir("website/files/blog").unwrap();

    blog_folder
        .filter_map(|f| f.ok())
        .filter(|f| f.file_name() != OsStr::new("template.html") && f.path().extension() == Some(OsStr::new("html")))
        .map(|html_file| (Cbmd::from_html_file(&html_file.path()), html_file.path()))
        .filter(|(data, _)| data.is_ok())
        .map(|(data, path)| (data.unwrap(), path))
        .for_each(|(data, mut path)| {
            path.set_extension("cbmd");
            data.write_to_file(&path).unwrap();
        });

    println!("done generating CBMD!");
}