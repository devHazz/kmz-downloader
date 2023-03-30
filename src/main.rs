use kmz_downloader::config::Config;
use kmz_downloader::scraper::Listing;
#[tokio::main]
async fn main() {
    let dir_url = Config::read().expect("could not read config").dir_url;
    if !dir_url.is_empty() {
        let listing = Listing::default().read(dir_url.to_string()).await.expect("could not get directory listing");
        println!("{:?}", listing);
        listing.records[1].download().await;
    } else {
        panic!("config directory url: empty")
    }
}
