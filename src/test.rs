use super::{
    from_directory,
    AssetStore,
};

#[test]
fn test_load() {
    let mut store = from_directory("./src/");
    store.load("test.rs");
    let loaded = store.fetch_block("test.rs");
    assert!(loaded.is_ok());
}

#[test]
fn test_load_all() {
    let mut store = from_directory("./src/");
    store.load_all(vec!["test.rs", "lib.rs"].into_iter());
    {
        let test = store.fetch_block("test.rs");
        assert!(test.is_ok());
    }
    {
        let lib = store.fetch_block("lib.rs");
        assert!(lib.is_ok());
    }
}

#[test]
fn test_load_fail() {
    let mut store = from_directory("./src/");
    store.load("foo.rs");
    let loaded = store.fetch_block("foo.rs");
    assert!(loaded.is_err());
}

#[test]
fn test_load_same() {
    let mut store = from_directory("./src/");
    store.load("foo.rs");
    store.load("foo.rs");
    let loaded = store.fetch_block("foo.rs");
    assert!(loaded.is_err());
}

#[test]
fn test_fetch_regular() {
    let mut store = from_directory("./src/");
    store.load("lib.rs");
    // woooo, busy loop!
    loop {
        match store.fetch("lib.rs") {
            Ok(Some(_)) => { break; }
            Ok(None) => { continue; }
            Err(_) => { assert!(false) }
        }
    }
}

#[test]
fn test_unload() {
    let mut store = from_directory("./src/");

    store.load("lib.rs");
    assert!(store.fetch_block("lib.rs").is_ok());

    store.unload("lib.rs");
    assert!(store.fetch_block("foo.rs").is_err());
}
