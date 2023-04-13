use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};

use geo_types::{GeometryCollection};
use kml::{ quick_collection, KmlReader};

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
            let mut file = archive.by_index(i).expect("could not get archive");
            if file.name().contains("/") {
                let folder: Vec<&str> = file.name().split("/").collect();
                fs::create_dir_all(format!("temp/{}/", folder[0]))
                    .expect("could not create directory");
            }
            if file.name().ends_with("kmz") {
                let mut out_archive = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(format!("temp/{}", file.name()))
                    .expect("could not create temp archive");
                io::copy(&mut file, &mut out_archive).unwrap();
                let mut reader =
                    KmlReader::<_, f64>::from_kmz_path(format!("temp/{}", file.name())).unwrap();
                let kml = reader.read().expect("could not read kml");
                let _doc: GeometryCollection<f64> = quick_collection(kml).unwrap();
            }
        }
    }
}
