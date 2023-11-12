use blake2::{Blake2s256, Digest};
use chrono::{Datelike, NaiveDateTime};
use data_encoding::HEXLOWER;
use fasthash::{
    city::crc::Hasher128 as CityCrcHasher128, murmur3::Hasher128_x64 as Murmur3Hasher128,
    FastHasher, HasherExt,
};
use rand::{distributions::Alphanumeric, Rng};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hasher;
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, io, path::PathBuf};

pub fn hash_file_blake2(path: PathBuf) -> String {
    let mut hasher = Blake2s256::new();
    if let Ok(mut f) = fs::File::open(path.as_path()) {
        io::copy(&mut f, &mut hasher).expect("failed to copy bytes to hasher");
        let hash = hasher.finalize();
        let s = HEXLOWER.encode(&hash[..]);
        s
    } else {
        panic!("Could not open file {:?}", path)
    }
}

pub fn hash_file_city128(path: PathBuf) -> String {
    let mut hasher = CityCrcHasher128::new();
    if let Ok(mut f) = fs::File::open(path.as_path()) {
        let mut input = vec![];
        f.read_to_end(&mut input).expect("error reading file data");
        hasher.write(&input);
        let hash = hasher.finish_ext();
        hash.to_string()
    } else {
        panic!("Could not open file {:?}", path)
    }
}

pub fn hash_file_murmur128(path: PathBuf) -> String {
    let mut hasher = Murmur3Hasher128::new();
    if let Ok(mut f) = fs::File::open(path.as_path()) {
        let mut input = vec![];
        f.read_to_end(&mut input).expect("error reading file data");
        hasher.write(&input);
        let hash = hasher.finish_ext();
        hash.to_string()
    } else {
        panic!("Could not open file {:?}", path)
    }
}

pub type ExifData = HashMap<String, String>;

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub file_path: String,
    pub hash: String,
    pub exif_data: Option<ExifData>,
}

impl FileInfo {
    pub fn get_exif_date(&self) -> Option<NaiveDateTime> {
        self.exif_data
            .as_ref()
            .and_then(|exif| {
                exif.get("Date and Time")
                    .or_else(|| exif.get("Date and Time (Digitized)"))
                    .or_else(|| exif.get("Date and Time (Original)"))
            })
            .and_then(|s| {
                if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S") {
                    Some(dt)
                } else {
                    None
                }
            })
    }
}

fn rnd_str(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub struct MoveOperation {
    source_file: FileInfo,
    target_path: PathBuf,
}

impl MoveOperation {
    pub fn new(target: PathBuf, fi: FileInfo) -> MoveOperation {
        let mut tp = target.clone();
        if let Some(date) = fi.get_exif_date() {
            tp.push("by_date");
            tp.push(date.year().to_string());
            tp.push(date.month().to_string());
            tp.push(date.day().to_string());
        } else {
            tp.push("unsorted");
        }
        let file_name = Path::new(&fi.file_path)
            .file_name()
            .expect("without a filename, no MoveOperation");
        tp.push(file_name);
        MoveOperation {
            source_file: fi,
            target_path: tp,
        }
    }

    pub fn execute(self) {
        let source_path = Path::new(&self.source_file.file_path);
        if !source_path.exists() || source_path.is_dir() {
            return;
        }
        let target_parent = self
            .target_path
            .parent()
            .expect("without parent we cannot move");
        if !target_parent.exists() {
            fs::create_dir_all(target_parent).expect("error creating parent dir")
        }

        let target_path = if self.target_path.exists() {
            let ext = self.target_path.extension().and_then(|e| e.to_str());
            let file_name = self.target_path.file_name().unwrap();
            let mut parent = target_parent.to_path_buf();
            let postfix = rnd_str(6);
            let file_name = if let Some(ext) = ext {
                format!(
                    "{}_duplicate_{}.{}",
                    file_name.to_str().unwrap(),
                    postfix,
                    ext
                )
            } else {
                format!("{}_duplicate_{}", file_name.to_str().unwrap(), postfix)
            };
            eprintln!(
                "warning: detected potential duplicate, renaming to: {}",
                file_name
            );
            parent.push(file_name);
            parent
        } else {
            self.target_path.clone()
        };

        if let Err(e) = fs::rename(&source_path, &target_path) {
            eprintln!(
                "error moving file: {} source: {:?} target: {:?}",
                e, source_path, target_path
            )
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::Path;

    use super::*;
    use chrono::{Datelike, Timelike};
    use serde_json::json;

    pub fn test_data() -> FileInfo {
        let fi = json!({
            "exif_data": {
                "Color Space": "sRGB",
                "Components Configuration": "Y Cb Cr -",
                "Compression": "JPEG compression",
                "Custom Rendered": "Normal process",
                "Date and Time": "2008:12:11 13:36:33",
                "Date and Time (Digitized)": "2008:12:11 13:36:33",
                "Date and Time (Original)": "2008:12:11 13:36:33",
                "Digital Zoom Ratio": "0.00",
                "Exif Version": "Exif Version 2.2",
                "Exposure Bias": "0.00 EV",
                "Exposure Mode": "Auto exposure",
                "Exposure Time": "1/250 sec.",
                "Flash": "Flash fired, auto mode",
                "FlashPixVersion": "FlashPix Version 1.0",
                "F-Number": "f/2.8",
                "Interoperability Index": "R98",
                "Interoperability Version": "0100",
                "ISO Speed Ratings": "400",
                "Light Source": "Unknown",
                "Manufacturer": "Sony Ericsson",
                "Metering Mode": "Center-weighted average",
                "Model": "K800i",
                "Orientation": "Top-left",
                "Pixel X Dimension": "2048",
                "Pixel Y Dimension": "1536",
                "Resolution Unit": "Inch",
                "Scene Capture Type": "Standard",
                "Software": "R1KG001     prgCXC1250210_GENERIC_W 0.0",
                "Subject Distance Range": "Distant view",
                "ThumbnailSize": "5431",
                "White Balance": "Auto white balance",
                "X-Resolution": "72",
                "YCbCr Positioning": "Co-sited",
                "Y-Resolution": "72",
            },
            "file_path": "/home/k/nextcloud/Photos/pics/2008/12/01/dsc04143.jpg",
            "hash": "275633487630418693378022278012862368828",
        });
        return serde_json::from_value(fi).unwrap();
    }

    #[test]
    fn extract_exif_date() {
        let fi = test_data();
        let date = fi.get_exif_date().unwrap();
        assert_eq!((date.year(), date.month(), date.day()), (2008, 12, 11));
        assert_eq!((date.hour(), date.minute(), date.second()), (13, 36, 33));
    }

    #[test]
    fn test_path_calculation() {
        let target_path = Path::new("/tmp/foo");
        let fi = test_data();
        let mo = MoveOperation::new(target_path.to_path_buf(), fi);
        assert_eq!(
            mo.target_path,
            Path::new("/tmp/foo/by_date/2008/12/11/dsc04143.jpg").to_path_buf()
        );
        assert_eq!(
            mo.target_path.as_path().parent().unwrap().to_path_buf(),
            Path::new("/tmp/foo/by_date/2008/12/11/").to_path_buf()
        )
    }
}
