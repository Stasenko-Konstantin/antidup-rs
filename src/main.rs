mod phash;

struct Pic {
    name: String,
    hash: String,
}

fn main() {
    let formats: Vec<&str> = vec!("png", "jpg", "jpeg");

    let path: &str;
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        path = args[1].as_str();
    } else {
        path = "./";
    }
    let files = std::fs::read_dir(path).unwrap()
        .filter(|f| !f.as_ref().unwrap()
            .metadata().unwrap()
            .is_dir());
    let pics: Vec<Pic> = vec!();
}
