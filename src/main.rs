#![feature(unwrap_infallible)]

extern crate core;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug};
use std::fs;
use std::fs::remove_file;
use std::process::exit;
use crate::phash::{find_distance, find_hash};

mod phash;

#[derive(Debug, Clone, Eq, Hash)]
struct Pic {
    name: String,
    hash: String,
    size: u64,
}

impl PartialEq for Pic {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Pic {
    fn find_size(self: Self) -> String {
        let size = self.size;
        match size {
            _ if size < 1024 => format!("{:.2}b", size),
            _ if size < 1048576 => format!("{:.2}kb", size / 1024),
            _ if size < 1073741824 => format!("{:.2}mb", size / 1024 / 1024),
            _ if size < 1099511627776 => format!("{:.2}gb", size / 1024 / 1024 / 1024),
            _ => format!("{}b", size)
        }
    }
}

fn main() {
    let path: &str;
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--help".to_string()) {
        println!("{}", "usage:\n\
        \t--help  \t-- prints the help\n\
        \t--quiet \t-- prints duplicates only\n\
        \t--rm    \t-- delete smallest duplicate");
        exit(0)
    }
    if args.len() == 2 {
        path = check_path(args[1].as_str());
    } else {
        path = "./";
    }
    let cmds = parse_args(args.clone());
    check(path, cmds);
}

fn check_path(path: &str) -> &str {
    if &path[0..2] == "--" {
        return "./";
    }
    return path
}

fn parse_args(args: Vec<String>) -> HashMap<&'static str, bool> {
    let mut cmds = HashMap::new();
    for arg in args {
        match arg.as_str() {
            "--quiet" => cmds.insert("quiet", true),
            "--rm" => cmds.insert("rm", true),
            _ => {
                cmds.insert("quiet", false);
                cmds.insert("rm", false)
            }
        };
    }
    return cmds
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
        if !cmds["quiet"] {
            println!("{}", name);
        }
        let hash = find_hash(name.clone())?;
        let size= f.metadata().unwrap().len();
        Some(Pic {name, hash, size })
    }).collect();
    let result = find_duplicates(pics);
    if result.len() == 0 {
        println!("{}: no duplicates found", dir);
    } else {
        let mut s = "".to_string();
        for (k, v) in result {
            if cmds["rm"] {
                del(k.clone(), v.clone());
            }
            s += format!("\t{}, {} -- {}, {}\n", k.name.clone(), k.find_size(), v.name.clone(), v.find_size()).as_str()
        }
        println!("{}:\n{}", dir, &s[..s.len()-1]);
    }
}

fn del(k: Pic, v: Pic) {
    let d: Pic;
    if k.size < v.size {
        d = k;
    } else {
        d = v;
    }
    remove_file(d.name.clone()).expect(format!("cant delete {}", d.name).as_str());
}

fn find_duplicates(pics: Vec<Option<Pic>>) -> Vec<(Pic, Pic)> {
    let mut result: Vec<(Pic, Pic)> = Vec::new();
    let mut dups: Vec<HashMap<Pic, Pic>> = Vec::new();
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
                dup.insert(p, comp);
            }
            if dup.len() > 0 {
                dups.push(dup.clone());
            }
        }
        i += 1;
    }
    for dup in dups {
        for (k, v) in dup {
            result.push((k, v));
        }
    }
    result
}