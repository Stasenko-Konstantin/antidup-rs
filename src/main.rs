use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;

mod phash;

struct Pic {
    name: String,
    hash: String,
}

fn main() {
    let formats: Vec<&OsStr> = vec!("png".as_ref() as &OsStr, "jpg".as_ref() as &OsStr, "jpeg".as_ref() as &OsStr);

    let path: &str;
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        path = args[1].as_str();
    } else {
        path = "./";
    }
    let files: Vec<fs::DirEntry> = std::path::Path::new(path).read_dir().unwrap()
        .filter(|f| f.as_ref().unwrap().path().extension().is_some()).filter( |f|
        formats.contains(&(f.as_ref().unwrap().path().extension().unwrap())))
        .map(|f| f.unwrap()).collect();
    println!("{:?}", files);
    let pics: Vec<Pic> = Vec::new();
}
