extern crate asset_store;

use asset_store::from_directory;
use asset_store::AssetStore;

pub fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_string()
}

fn main() {
    // Make a new asset store from this examples directory.
    let store = from_directory("./examples/");
    // Asynchronously load this file.
    store.load("basic_file.rs");

    // Block until the file is loaded.
    let contents = store.map_resource_block("basic_file.rs", to_string);
    // Print the bytes of the file.
    println!("{}", contents);
}
