use clap::Parser;
use dedupe_utils::{hash_file_city128, ExifData, FileInfo};
use globwalker::DirEntry;
use rayon::prelude::*;
use std::{path::PathBuf, process::Command};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    dir: String,
}

fn extract_exif(path: PathBuf) -> Option<ExifData> {
    let cmd = Command::new("exif")
        .arg("-m")
        .arg(&path)
        .output()
        .expect("Could not run exif");
    if cmd.status.success() {
        let data = String::from_utf8_lossy(&cmd.stdout)
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| line.split('\t').collect::<Vec<&str>>())
            .filter(|pair| pair.len() == 2)
            .map(|pair| {
                let name = pair[0].trim().to_owned();
                let value = pair[1].trim().to_owned();
                (name, value)
            })
            .collect::<ExifData>();
        return Some(data);
    }
    None
}

fn main() {
    let args = Args::parse();
    let pattern = format!("{}/**/*", args.dir);

    let files = globwalker::glob(pattern)
        .unwrap()
        .filter(|file| {
            if let Ok(file) = file {
                if file.path().is_dir() {
                    return false;
                }
                true
            } else {
                false
            }
        })
        .map(|file| file.unwrap())
        .collect::<Vec<DirEntry>>();

    files
        .par_iter()
        .map(|f| FileInfo {
            file_path: f.path().to_string_lossy().into_owned(),
            hash: hash_file_city128(f.path().to_path_buf()),
            exif_data: extract_exif(f.path().to_path_buf()),
        })
        .for_each(|file_info| {
            println!(
                "{}",
                serde_json::to_string(&file_info).expect("json encoding failed")
            )
        })
}
