# Asset Store [![Build Status](https://travis-ci.org/PistonDevelopers/asset_store.svg?branch=master)](https://travis-ci.org/PistonDevelopers/asset_store)

[Api Documentation](http://tyoverby.com/asset_store/asset_store/trait.AssetStore.html)

A unified method for easily reading and caching files over the filesystem
and network.

Calls to `load()` process asynchronously, so it is possible to load files
from different sources in parallel.

### Read files from disk

When reading a files out of a directory store, it is impossible to read outside
of the directory specified.

```rust
extern crate asset_store;

use asset_store::from_directory;
use asset_store::AssetStore;

fn to_string(bytes: &[u8]) -> String {
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

```

### Read files over http

You can also read files off of a web server.

```rust
extern crate asset_store;

use asset_store::from_url;
use asset_store::AssetStore;

fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_string()
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

```

### Read files out of memory

If your files are small enough, you can bundle them into your binary by using
[resources-package](https://github.com/tomaka/rust-package.git).

```rust
#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;
extern crate resources_package_package;
extern crate asset_store;

use resources_package_package::Package;

use asset_store::StaticStore;
use asset_store::AssetStore;

fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_string()
}

// Store all .rs files in the examples directory in the
// binary during compilation
static package: Package =
    resources_package!([
        "./"
    ]
);

fn main() {
    // Use an in memory store.
    let store = StaticStore::new(&package);

    // Load the file right out of memory.
    let stat = store.map_resource_block("static_resources.rs", to_string);
    println!("{}", stat.unwrap());
}

```

### Combine different stores into one.

Having multiple stores laying around for different sources is a pain, so
by combining them into one and using prefixes you can access many
file stores of different types.

```rust
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

```
