extern crate asset_store;

use asset_store::from_url;
use asset_store::AssetStore;

fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn main() {
    // Make a new asset store with google as the root
    let store = from_url("http://www.google.com/");
    // Asynchronously load this file.
    store.load("basic_file.rs");

    // Block until the file is loaded.
    let contents = store.map_resource_block("robots.txt", to_string);
    // Print the bytes of the file.
    println!("{}", contents.unwrap());
}
