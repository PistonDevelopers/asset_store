extern crate asset_store;

use asset_store::from_url;
use asset_store::from_directory;
use asset_store::AssetStore;
use asset_store::MultiStore;

fn id<A>(a:A) -> A { a }

fn main() {
    // Create a file store for the local file system
    let file_store = from_directory("./examples/");
    // Create a file store for the google
    let web_store = from_url("http://www.google.com/");

    // Make a MultiStore to combine all our other storage methods
    let mut combo = MultiStore::new();
    combo.add("web", web_store, id);
    combo.add("file", file_store, id);

    combo.load("file:multi.rs");
    combo.load("web:robots.txt");

    {
        let robots = combo.fetch_block("web:robots.txt");
        println!("{}", String::from_utf8_lossy(robots.unwrap()));
    } {
        let multi = combo.fetch_block("file:multi.rs");
        println!("{}", String::from_utf8_lossy(multi.unwrap()));
    }
}

