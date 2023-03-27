use anyhow::Result;
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub struct Record {
    pub kind: RecordType,
    pub name: String,
    pub file_size: String,
}

impl Record {
    pub fn get_type(row: ElementRef) -> Option<RecordType> {
        let kind_data_cell = row.children().nth(0);
        if let Some(kind) = kind_data_cell {
            let kind_elem = kind
                .value()
                .as_element()
                .expect("could not get data cell type");
            if kind_elem.attrs().any(|a| a.0 == "valign" && a.1 == "top") {
                let kind_child = kind.children().nth(0).unwrap();
                return Some(get_record_type(
                    kind_child
                        .value()
                        .as_element()
                        .unwrap()
                        .attr("alt")
                        .unwrap_or("[   ]"),
                ));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn get_name(row: ElementRef) -> Option<String> {
        let name_data_cell = row.children().nth(1);
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

    pub fn get_size(row: ElementRef) -> Option<String> {
        let size_data_cell = row.children().nth(3);
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

#[derive(Debug)]
pub enum RecordType {
    ParentDirectory,
    Directory,
    File,
    Unknown,
    None,
}

fn get_record_type(alt: &str) -> RecordType {
    let record_type = match alt {
        "[DIR]" => RecordType::Directory,
        "[PARENTDIR]" => RecordType::ParentDirectory,
        "[TXT]" | "[IMG]" | "[VID]" => RecordType::File,
        "[   ]" => RecordType::Unknown,
        _ => RecordType::None,
    };
    return record_type;
}

#[derive(Default)]
pub struct Listing {
    pub is_root: bool,
    pub records: Vec<Record>,
}

impl Listing {
    pub async fn read(&self, uri: String) -> Result<()> {
        let body = reqwest::get(uri).await?.text().await?;
        let records = self.read_records(body);
        println!("{:?}", records);
        Ok(())
    }
    pub fn read_record(&self, row: ElementRef) -> Record {
        let kind = Record::get_type(row).expect("could not get record kind");
        let name = Record::get_name(row).expect("could not get record name");
        let file_size = Record::get_size(row).expect("could not get record file size");
        return Record {
            kind,
            name,
            file_size,
        };
    }
    pub fn read_records(&self, body: String) -> Result<Vec<Record>> {
        let html = Html::parse_document(&body);
        let table_selector = Selector::parse("body > table > tbody").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
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
                records.push(record);
            }
        }
        Ok(records)
    }
}
