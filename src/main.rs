#![feature(unwrap_infallible)]

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug};
use std::fs;
use std::ops::Add;
use std::process::exit;
use crate::phash::{find_distance, find_hash};

mod phash;

#[derive(Debug, Clone)]
struct Pic {
    name: String,
    hash: String,
}

impl PartialEq for Pic {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

fn main() {
    let path: &str;
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--help".to_string()) {
        println!("{}", "usage:\n\
        \t--help  \t-- prints the help\n\
        \t--quiet \t-- prints duplicates only");
        exit(0)
    }
    if args.len() == 2 {
        path = args[1].as_str();
    } else {
        path = "./";
    }
    let mut cmds = HashMap::new();
    if args.contains(&"--quiet".to_string()) {
        cmds.insert("quiet", true);
    } else {
        cmds.insert("quiet", false);
    }
    check(path, cmds);
}

fn check(dir: &str, cmds: HashMap<&str, bool>) {
    let formats: Vec<&OsStr> = vec!("png".as_ref() as &OsStr,
                                      "jpg".as_ref() as &OsStr,
                                      "jpeg".as_ref() as &OsStr);
    let files: Vec<fs::DirEntry> = std::path::Path::new(dir).read_dir().unwrap()
        .filter(|f| f.as_ref().unwrap().path().extension().is_some()).filter(|f|
        formats.contains(&(f.as_ref().unwrap().path().extension().unwrap())))
        .map(|f| f.unwrap()).collect();
    if files.len() < 2 {
        println!("{}: no duplicates found", dir);
        exit(0);
    }
    println!("calculation...");
    let pics: Vec<Option<Pic>> = files.into_iter().map(|f| {
        let name = f.path().file_name().unwrap().to_string_lossy().chars().as_str().to_string();
        if !cmds.get("quiet").unwrap() {
            println!("{}", name);
        }
        let hash = find_hash(name.clone())?;
        let size = find_size(f.metadata().unwrap().len());
        Some(Pic {name: format!("{}, {}", name, size), hash })
    }).collect();
    let result = find_duplicates(pics);
    if result == "" {
        println!("{}: no duplicates found", dir);
    } else {
        println!("{}:\n{}", dir, result[..result.len()-1].to_string());
    }
}

fn find_size(size: u64) -> String {
    match size {
        _ if size < 1024 => format!("{:.2}b", size),
        _ if size < 1048576 => format!("{:.2}kb", size/1024),
        _ if size < 1073741824 => format!("{:.2}mb", size/1024/1024),
        _ if size < 1099511627776 => format!("{:.2}gb", size/1024/1024/1024),
        _  => format!("{}b", size)
    }
}

fn find_duplicates(pics: Vec<Option<Pic>>) -> String {
    let mut result = String::new();
    let mut dups: Vec<HashMap<String, String>> = Vec::new();
    let mut i = 0;
    for p in pics.clone() {
        let p = if p.is_some() { p.unwrap() } else { continue };
        let s: Vec<Option<Pic>>;
        if pics.len() == 1 {
            s = pics[i..].to_vec();
        } else {
            s = pics[i +1..].to_vec();
        }
        for comp in s {
            let comp = if comp.is_some() { comp.unwrap() } else { continue };
            let p = p.clone();
            let mut dup = HashMap::new();
            let distance = find_distance(&p.hash.chars(), &comp.hash.chars());
            if distance < 3 {
                dup.insert(p.name, comp.name);
            }
            if dup.len() > 0 {
                dups.push(dup.clone());
            }
        }
        i += 1;
    }
    for dup in dups {
        for (k, v) in dup {
            result = result.add(format!("\t{} -- {}\n", k, v).as_str());
        }
    }
    result
}