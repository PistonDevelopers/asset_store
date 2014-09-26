
use super::{
    from_directory,
    AssetStore,
};

fn to_unit<A>(_: A) -> () {()}

#[test]
fn test_load() {
    let store = from_directory("./src/");
    store.load("test.rs");
    let loaded = store.map_resource_block("test.rs", |x| to_unit(x));
    assert!(loaded.is_ok());
}

#[test]
fn test_load_all() {
    let store = from_directory("./src/");
    store.load_all(vec!["test.rs", "lib.rs"].into_iter());
    {
        let test = store.map_resource_block("test.rs", |x| to_unit(x));
        assert!(test.is_ok());
    }
    {
        let lib = store.map_resource_block("lib.rs", |x| to_unit(x));
        assert!(lib.is_ok());
    }
}

#[test]
fn test_load_fail() {
    let store = from_directory("./src/");
    store.load("foo.rs");
    let loaded = store.map_resource_block("foo.rs", |x| to_unit(x));
    assert!(loaded.is_err());
}

#[test]
fn test_load_same() {
    let store = from_directory("./src/");
    store.load("foo.rs");
    store.load("foo.rs");
    let loaded = store.map_resource_block("foo.rs", |x| to_unit(x));
    assert!(loaded.is_err());
}

#[test]
fn test_fetch_regular() {
    let store = from_directory("./src/");
    store.load("lib.rs");
    // woooo, busy loop!
    loop {
        match store.map_resource("lib.rs", |x| to_unit(x)) {
            Ok(Some(_)) => { break; }
            Ok(None) => { continue; }
            Err(_) => { assert!(false) }
        }
    }
}

#[test]
fn test_unload() {
    let store = from_directory("./src/");

    store.load("lib.rs");
    assert!(store.map_resource_block("lib.rs", |x| to_unit(x)).is_ok());
    assert!(store.map_resource("lib.rs", |x| to_unit(x)).is_ok());

    store.unload("lib.rs");
    match store.map_resource("lib.rs", |x| to_unit(x)) {
        Ok(None) => assert!(true),
        _ => assert!(false)
    }
}
