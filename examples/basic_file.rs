extern crate asset_store;

use asset_store::from_directory;
use asset_store::AssetStore;

fn main() {
    // Make a new asset store from this examples directory.
    let mut store = from_directory("./examples/");
    // Asynchronously load this file.
    store.load("basic_file.rs");

    // Block until the file is loaded.
    let bytes = store.fetch_block("basic_file.rs");
    // Print the bytes of the file.
    println!("{}", String::from_utf8_lossy(bytes.unwrap()));
}
