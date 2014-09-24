#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;
extern crate asset_store;

use asset_store::StaticStore;
use asset_store::AssetStore;

// Store all .rs files in the examples directory in the
// binary during compilation
static package: &'static [(&'static [u8], &'static [u8])] =
    resources_package!([
        "./*.rs"
    ]
);

fn main() {
    // Use an in memory store.
    let mut store = StaticStore::new(package);

    // Load the file right out of memory.
    let stat = store.fetch_block("static_resources.rs");
    println!("{}", String::from_utf8_lossy(stat.unwrap()));
}
