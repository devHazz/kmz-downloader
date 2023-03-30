use anyhow::Result;
use std::fs;
use yaml_rust::YamlLoader;

pub struct Config {
   pub dir_url: String
}

impl Config {
    pub fn read() -> Result<Self> {
        let s = fs::read_to_string("config.yml").expect("could not read config file");
        let config = YamlLoader::load_from_str(&s).unwrap();
        let dir_url = config[0]["dir_url"].as_str().expect("could not get kmz directory url").to_string();
        Ok(Config { dir_url })
    }
}