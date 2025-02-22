#![feature(unwrap_infallible)]

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug};
use std::fs::{self, remove_file};
use std::path::PathBuf;
use std::process::exit;
use clap::Parser;
use crate::phash::*;

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
}

impl Pic {
    fn find_size(self) -> String {
        const KB: u64 = 1024;
        let size = self.size;
        match size {
            _ if size < KB => format!("{:.2}b", size),
            _ if size < u64::pow(KB, 2) => format!("{:.2}kb", size / 1024),
            _ if size < u64::pow(KB, 3) => format!("{:.2}mb", size / 1024 / 1024),
            _ if size < u64::pow(KB, 4) => format!("{:.2}gb", size / 1024 / 1024 / 1024),
            _ => format!("{}b", size)
        }
    }
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Recursive {
    #[default]
    Non,
    Segmented,
    Flat
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    quiet: bool,
    
    #[arg(long)]
    rm: bool,
    
    #[arg(short, long, default_value_t, value_enum)]
    recursive: Recursive,
    
    #[arg(short, long)]
    deep: u32,
    
    #[arg(short, long)]
    path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    check(if args.path.is_some() {
        args.path.clone().unwrap()
    } else {
        PathBuf::from("./")
    }, args);
}

fn mk_file_index(dir: PathBuf, rec: Recursive) {
    let formats: Vec<&OsStr> = vec!("png".as_ref(), "jpg".as_ref(), "jpeg".as_ref());
    let files: Vec<fs::DirEntry> =
        dir.read_dir().unwrap_or_else(|_| panic!("cant read path: {}", dir.display())).
            filter(|f|
                f.as_ref().unwrap().path().extension().is_some() &&
                    formats.contains(&(f.as_ref().unwrap().path().extension().unwrap()))).
            map(|f| f.unwrap()).collect();
    if files.len() < 2 {
        println!("{:?}: no duplicates found", dir);
        exit(0);
    }
}

fn check(dir: PathBuf, args: Args) {
    let files = mk_file_index(dir, args.recursive);
    
    println!("calculation...");
    let pics: Vec<Option<Pic>> = files.into_iter().map(|f| {
        let name = f.path().file_name()?.to_string_lossy().chars().as_str().to_string();
        if !args.quiet {
            println!("{}", name);
        }
        let hash = find_hash(name.clone())?;
        let size= f.metadata().unwrap().len();
        Some(Pic {name, hash, size})
    }).collect();
    
    let result = find_duplicates(pics);
    if result.is_empty() {
        println!("{:?}: no duplicates found", dir);
    } else {
        let mut s = "".to_string();
        for (k, v) in result {
            if args.rm {
                let d = if k.size < v.size { k.clone() } else { v.clone() };
                remove_file(d.name.clone()).unwrap_or_else(|_| panic!("cant delete {}", d.name));
            }
            s += format!("\t{}, {} -- {}, {}\n", 
                         k.name.clone(), k.find_size(), 
                         v.name.clone(), v.find_size()).as_str()
        }
        println!("{:?}:\n{}", dir, &s[..s.len()-1]);
    }
}

fn find_duplicates(pics: Vec<Option<Pic>>) -> Vec<(Pic, Pic)> {
    let mut result: Vec<(Pic, Pic)> = Vec::new();
    let mut dups: Vec<HashMap<Pic, Pic>> = Vec::new();
    let mut i = 0;
    for p in pics.clone() {
        let p = if p.is_some() { p.unwrap() } else { continue };
        let s = if pics.len() == 1 {
            pics[i..].to_vec()
        } else {
            pics[i+1..].to_vec()
        };
        for comp in s {
            let comp = if comp.is_some() { comp.unwrap() } else { continue };
            let p = p.clone();
            let mut dup = HashMap::new();
            let distance = find_distance(&p.hash.chars(), &comp.hash.chars());
            if distance < MIN_DISTANCE {
                dup.insert(p, comp);
            }
            if !dup.is_empty() {
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