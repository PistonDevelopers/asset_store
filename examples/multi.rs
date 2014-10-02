extern crate asset_store;

use asset_store::from_url;
use asset_store::from_directory;
use asset_store::AssetStore;
use asset_store::MultiStore;

fn id<A>(a:A) -> A { a }

fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_string()
}

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
        let robots = combo.map_resource_block("web:robots.txt", to_string);
        println!("{}", robots.unwrap());
    } {
        let multi = combo.map_resource_block("file:multi.rs", to_string);
        println!("{}", multi.unwrap());
    }

}
