use kmz_downloader::scraper::Listing;
use yaml_rust::YamlLoader;
use std::fs;
#[tokio::main]
async fn main() {
    let s = fs::read_to_string("config.yml").expect("could not read config file");
    let config = YamlLoader::load_from_str(&s).unwrap();
    let dir_url = config[0]["dir_url"].as_str().expect("could not get kmz directory url");
    if !dir_url.is_empty() {
        Listing::default().read(dir_url.to_string()).await;
    } else {
        panic!("config directory url: empty")
    }
}
