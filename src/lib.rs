use blake2::{Blake2s256, Digest};
use data_encoding::HEXLOWER;
use fasthash::{
    city::crc::Hasher128 as CityCrcHasher128, murmur3::Hasher128_x64 as Murmur3Hasher128,
    FastHasher, HasherExt,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hasher;
use std::io::Read;
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
