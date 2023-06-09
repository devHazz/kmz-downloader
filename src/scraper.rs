use std::{fs, path::Path};

use anyhow::Result;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::str::FromStr;
use thiserror::Error;

use crate::{config::Config, kmz::CompressedKMZ};

const SCRAPER_BODY_SELECTOR: &str = "body > table > tbody";

#[derive(Debug)]
pub struct Record {
    pub kind: RecordType,
    pub uri: String,
    pub name: String,
    pub file_size: String,
}

pub trait RecordRef {
    fn name(&self) -> Option<String>;
    fn kind(&self) -> Option<RecordType>;
    fn size(&self) -> Option<String>;
}

impl RecordRef for ElementRef<'_> {
    fn name(&self) -> Option<String> {
        let name_data_cell = self.children().nth(1);
        if let Some(name) = name_data_cell {
            let name_child = name.children().nth(0).unwrap();
            if let Some(text_child) = name_child.first_child() {
                return match text_child.value().is_text() {
                    true => Some(text_child.value().as_text().unwrap().to_string()),
                    false => None,
                };
            } else {
                None
            }
        } else {
            None
        }
    }

    fn kind(&self) -> Option<RecordType> {
        let kind_data_cell = self.children().nth(0);
        if let Some(kind) = kind_data_cell {
            let kind_elem = kind
                .value()
                .as_element()
                .expect("could not get data cell type");
            if kind_elem.attrs().any(|(e, v)| e == "valign" && v == "top") {
                let kind_child = kind.children().nth(0).unwrap();
                Some(
                    RecordType::from_str(
                        kind_child
                            .value()
                            .as_element()
                            .unwrap()
                            .attr("alt")
                            .unwrap_or(RecordType::Unknown.to_string().as_str()),
                    )
                    .unwrap(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    fn size(&self) -> Option<String> {
        let size_data_cell = self.children().nth(3);
        if let Some(size) = size_data_cell {
            let size_data_child = size.first_child().unwrap().value();
            return match size_data_child.is_text() {
                true => Some(size_data_child.as_text().unwrap().to_string()),
                false => None,
            };
        } else {
            None
        }
    }
}

impl Record {
    pub fn download(&self) {
        if !self.uri.is_empty()
            && !(matches!(self.kind, RecordType::ParentDirectory)
                || matches!(self.kind, RecordType::Directory))
        {
            let file = reqwest::blocking::get(&self.uri)
                .expect("could not get archive file")
                .bytes()
                .expect("could not get archive body");
            println!("downloading: {}", self.name);
            if !(Path::new("temp/").exists()) {
                fs::create_dir_all("temp/").expect("could not create temp directory");
            }
            fs::write(format!("temp/{}", self.name), file).expect("could not write record to file");
        }
    }

    #[warn(dead_code)]
    fn downloaded(&self) -> bool {
        return Path::new(&format!("temp/{}", self.name)).exists();
    }

    pub fn is_kmz(&self) -> bool {
        let r = Regex::new(r"(RE_)|(KMZ)|(RSA-DATA).*\.zip").unwrap();
        return r.is_match(&self.name);
    }
    pub fn as_kmz(&self) -> Option<CompressedKMZ> {
        return match self.is_kmz() {
            true => CompressedKMZ::new(format!("temp/{}", self.name.to_owned())),
            false => None,
        };
    }
}

#[derive(Error, Debug)]
pub enum RecordTypeError {
    #[error("invalid record type: {0}")]
    InvalidRecordType(String),
}

#[derive(Debug)]
pub enum RecordType {
    ParentDirectory,
    Directory,
    TextFile,
    ImageFile,
    VideoFile,
    Unknown,
}

impl FromStr for RecordType {
    type Err = RecordTypeError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "[DIR]" => Ok(RecordType::Directory),
            "[PARENTDIR]" => Ok(RecordType::ParentDirectory),
            "[TXT]" => Ok(RecordType::TextFile),
            "[IMG]" => Ok(RecordType::ImageFile),
            "[VID]" => Ok(RecordType::VideoFile),
            "[   ]" => Ok(RecordType::Unknown),
            e => Err(RecordTypeError::InvalidRecordType(e.to_string())),
        }
    }
}

impl ToString for RecordType {
    fn to_string(&self) -> String {
        match self {
            RecordType::Directory => "[DIR]".to_string(),
            RecordType::ParentDirectory => "[PARENTDIR]".to_string(),
            RecordType::TextFile => "[TXT]".to_string(),
            RecordType::ImageFile => "[IMG]".to_string(),
            RecordType::VideoFile => "[VID]".to_string(),
            RecordType::Unknown => "[   ]".to_string(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Listing {
    pub is_root: bool,
    pub records: Vec<Record>,
}

impl Listing {
    pub fn read(&self, uri: String) -> Result<Self> {
        let body = reqwest::blocking::get(uri)?.text()?;
        let records = self
            .read_records(body)?;
        let is_root = !records
            .iter()
            .any(|r| matches!(r.kind, RecordType::ParentDirectory));

        Ok(Listing { is_root, records })
    }
    fn read_record(&self, row: ElementRef) -> Record {
        let kind = row.kind().expect("could not get record kind");
        let name = row.name().expect("could not get record name");
        let file_size = row.size().expect("could not get record file size");
        let uri = Config::read().expect("could not read config").dir_url + &name;
        return Record {
            kind,
            uri,
            name,
            file_size,
        };
    }
    pub fn read_records(&self, body: String) -> Result<Vec<Record>> {
        let html = Html::parse_document(&body);
        let table_selector =
            Selector::parse(SCRAPER_BODY_SELECTOR).expect("could not select table body");
        let tr_selector = Selector::parse("tr").expect("could not select table rows");
        let mut records: Vec<Record> = Vec::new();

        let rows = html
            .select(&table_selector)
            .next()
            .expect("could not select table");
        for row in rows.select(&tr_selector) {
            let row = match row
                .children()
                .any(|child| child.value().as_element().unwrap().name() == "td")
            {
                true => Some(self.read_record(row)),
                false => None,
            };
            if let Some(record) = row {
                if record.is_kmz() {
                    record.download();
                    match record.as_kmz() {
                        Some(k) => k.unpack(),
                        None => println!("could not convert file to kmz archive"),
                    }
                    records.push(record);
                }
            }
        }
        Ok(records)
    }
}
