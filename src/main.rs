#![feature(core_intrinsics)]

use std::ffi::OsStr;
use std::fmt::{Debug};
use std::fs;
use crate::phash::find_hash;

mod phash;

#[derive(Debug)]
struct Pic {
    name: String,
    hash: String,
}

fn main() {
    let path: &str;
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        path = args[1].as_str();
    } else {
        path = "./";
    }
    check(path);
}

fn check(dir: &str) {
    let formats: Vec<&OsStr> = vec!("png".as_ref() as &OsStr,
                                      "jpg".as_ref() as &OsStr,
                                      "jpeg".as_ref() as &OsStr);
    let files: Vec<fs::DirEntry> = std::path::Path::new(dir).read_dir().unwrap()
        .filter(|f| f.as_ref().unwrap().path().extension().is_some()).filter(|f|
        formats.contains(&(f.as_ref().unwrap().path().extension().unwrap())))
        .map(|f| f.unwrap()).collect();
    if files.len() < 2 {
        println!("{}: no duplicates find", dir);
    }
    let pics: Vec<Pic> = files.into_iter().map(|f| unsafe {
        let name = f.path().file_name().unwrap().to_string_lossy().chars().as_str().to_string();
        let hash = find_hash(name.clone());
        let size = 0; // get_size(f.path().file_name().unwrap());
        Pic {name: format!("{}, {}", name, size), hash }
    }).collect();
    println!("{:?}", pics);
}