use clap::Parser;
use dedupe_utils::{FileInfo, MoveOperation};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    target_dir: String,

    #[arg(short, long)]
    file_list: String,
}

fn load_file_infos(path: PathBuf) -> Vec<FileInfo> {
    let mut out = vec![];
    let file = File::open(path).expect("input file not found");
    let mut reader = BufReader::new(file).lines();
    while let Some(line) = reader.next() {
        match line {
            Ok(ref s) => {
                if let Ok(p) = serde_json::from_str(&s) {
                    out.push(p)
                }
            }
            _ => (),
        };
    }
    out
}

fn main() {
    let args = Args::parse();
    let source = Path::new(&args.file_list);
    let mut file_infos = load_file_infos(source.to_path_buf());

    let total = file_infos.len();
    let with_date = file_infos
        .par_iter()
        .fold(
            || 0,
            |acc, fi| {
                if let Some(exif) = &fi.exif_data {
                    if exif.contains_key("Date and Time")
                        || exif.contains_key("Date and Time (Digitized)")
                        || exif.contains_key("Date and Time (Original)")
                    {
                        acc + 1
                    } else {
                        acc
                    }
                } else {
                    acc
                }
            },
        )
        .reduce(|| 0, |a, b| a + b);
    println!("total: {} with exif date: {}", total, with_date);

    let target_dir = Path::new(&args.target_dir);
    file_infos.par_drain(..).for_each(|fi| {
        let mo = MoveOperation::new(target_dir.to_path_buf(), fi);
        mo.execute()
    })
}
