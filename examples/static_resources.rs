#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;
extern crate asset_store;

use asset_store::StaticStore;
use asset_store::AssetStore;

pub fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_string()
}

// Store all .rs files in the examples directory in the
// binary during compilation
static package: &'static [(&'static [u8], &'static [u8])] =
    resources_package!([
        "./*.rs"
    ]
);

fn main() {
    // Use an in memory store.
    let store = StaticStore::new(package);

    // Load the file right out of memory.
    let stat = store.map_resource_block("static_resources.rs", to_string);
    println!("{}", stat.unwrap());
}
