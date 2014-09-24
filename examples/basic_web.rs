extern crate asset_store;

use asset_store::from_url;
use asset_store::AssetStore;

fn main() {
    // Make a new asset store with google as the root
    let mut store = from_url("http://www.google.com/");
    // Asynchronously load this file.
    store.load("basic_file.rs");

    // Block until the file is loaded.
    let bytes = store.fetch_block("robots.txt");
    // Print the bytes of the file.
    println!("{}", String::from_utf8_lossy(bytes.unwrap()));
}
