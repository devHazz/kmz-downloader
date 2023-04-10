use std::{
    fs::{self, File},
    path::Path,
    str::Bytes, io,
};

use anyhow::Result;

pub struct CompressedKMZ {
    path: String,
}

impl CompressedKMZ {
    pub fn new(path: String) -> Option<CompressedKMZ> {
        if !path.is_empty() && Path::new(&path).exists() {
            Some(CompressedKMZ { path })
        } else {
            None
        }
    }
    pub fn unpack(&self) {
            let file = match File::open(&self.path) {
                Ok(file) => file,
                Err(e) => panic!("file create error: {}", e),
            };
            let mut archive = zip::ZipArchive::new(file).expect("could not parse zip archive");
            for i in 0..archive.len() {
                let file = archive.by_index(i).expect("could not get archive");
                println!("{}", file.name());
                let kmz_archive = zip::ZipArchive::new(file).expect("could not read kmz archive");
            }
    }
}
