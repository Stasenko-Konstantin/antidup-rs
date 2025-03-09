#![feature(unwrap_infallible)]

use crate::phash::*;
use clap::Parser;
use rayon::prelude::*;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::{remove_file, DirEntry};
use std::path::PathBuf;
use std::process::exit;

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
            _ => format!("{}b", size),
        }
    }
}

#[derive(clap::ValueEnum, Clone, PartialEq, Default, Debug)]
enum Recursive {
    #[default]
    Non,
    Segmented,
    Flat,
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

    #[arg(short, long, default_value_t = 0)]
    deep: u32,

    #[arg(short, long)]
    path: Option<PathBuf>,
}

fn main() {
    let mut args = Args::parse();
    if args.recursive == Recursive::Non {
        args.deep = 1;
    }
    check(
        if args.path.is_some() {
            args.path.clone().unwrap()
        } else {
            PathBuf::from("./")
        },
        args,
    );
}

fn read_dir(dir: PathBuf) -> Vec<DirEntry> {
    let formats: Vec<&OsStr> = vec!["png".as_ref(), "jpg".as_ref(), "jpeg".as_ref()];
    dir.clone()
        .read_dir()
        .unwrap_or_else(|_| panic!("cant read path: {}", dir.display()))
        .filter(|f| {
            f.as_ref().unwrap().path().extension().is_some()
                && formats.contains(&(f.as_ref().unwrap().path().extension().unwrap()))
        })
        .map(|f| f.unwrap())
        .collect()
}

fn mk_file_index(
    res: &mut HashMap<PathBuf, Vec<DirEntry>>,
    root: PathBuf,
    dir: PathBuf,
    is_flat: bool,
    deep: u32,
) {
    let mut files: Vec<DirEntry> = read_dir(dir.clone());
    if files.len() < 2 {
        println!("{:?}: no duplicates found", dir);
        exit(0);
    }

    if deep == 0 {
        res.insert(dir, files);
    } else {
        if !is_flat {
            res.insert(dir.clone(), files);
        } else if !res.contains_key(&dir) && is_flat {
            res.insert(root, files);
        } else {
            res.get_mut(&dir).unwrap().append(&mut files);
        }
        let _ = dir
            .read_dir()
            .unwrap_or_else(|_| panic!("cant read path: {}", dir.display()))
            .filter(|e| e.as_ref().unwrap().path().is_dir())
            .map(|d| {
                mk_file_index(
                    res,
                    dir.clone(),
                    d.unwrap_or_else(|_| panic!("cant read path: {}", dir.display()))
                        .path(),
                    is_flat,
                    deep - 1,
                )
            });
    }
}

fn check(dir: PathBuf, args: Args) {
    let is_flat = matches!(args.recursive, Recursive::Flat);
    let mut files: HashMap<PathBuf, Vec<DirEntry>> = HashMap::new();
    mk_file_index(&mut files, dir.clone(), dir.clone(), is_flat, args.deep);

    println!("calculation...");

    let mut pics: HashMap<&PathBuf, Vec<Option<Pic>>> = HashMap::new();
    for (k, v) in files.iter() {
        pics.insert(
            k,
            v.par_iter()
                .map(|e| {
                    let name = e
                        .path()
                        .file_name()?
                        .to_string_lossy()
                        .chars()
                        .as_str()
                        .to_string();
                    if !args.quiet {
                        println!("{}", name);
                    }
                    let hash = find_hash(name.clone())?;
                    let size = e.metadata().unwrap().len();
                    Some(Pic { name, hash, size })
                })
                .collect(),
        );
    }

    pics.par_iter()
        .for_each(|(dir, pics)| process_pics(dir, pics, args.rm))
}

fn process_pics(dir: &PathBuf, pics: &[Option<Pic>], rm: bool) {
    let result = find_duplicates(pics);
    if result.is_empty() {
        println!("{:?}: no duplicates found", dir);
    } else {
        let mut s = "".to_string();
        for (k, v) in result {
            if rm {
                let d = if k.size < v.size {
                    k.clone()
                } else {
                    v.clone()
                };
                remove_file(d.name.clone()).unwrap_or_else(|_| panic!("cant delete {}", d.name));
            }
            s += format!(
                "\t{}, {} -- {}, {}\n",
                k.name.clone(),
                k.find_size(),
                v.name.clone(),
                v.find_size()
            )
            .as_str()
        }
        println!("{:?}:\n{}", dir, &s[..s.len() - 1]);
    }
}

fn find_duplicates(pics: &[Option<Pic>]) -> Vec<(Pic, Pic)> {
    let mut result: Vec<(Pic, Pic)> = Vec::new();
    let mut dups: Vec<HashMap<Pic, Pic>> = Vec::new();
    let mut i = 0;
    for p in pics {
        let p = if p.is_some() {
            p.clone().unwrap()
        } else {
            continue;
        };
        let s = if pics.len() == 1 {
            pics[i..].to_vec()
        } else {
            pics[i + 1..].to_vec()
        };
        for comp in s {
            let comp = if comp.is_some() {
                comp.unwrap()
            } else {
                continue;
            };
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
