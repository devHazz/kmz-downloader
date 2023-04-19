use kmz_downloader::{scraper::Listing, config::Config};
 fn main() {
    let dir_url = Config::read().expect("could not read config").dir_url;
    if !dir_url.is_empty() {
        let _listing = Listing::default().read(dir_url.to_string());
        //println!("{:?}", listing);
    } else {
        panic!("config directory url: empty")
    }
}
