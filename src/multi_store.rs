use std::collections::HashMap;

use super::AssetStore;
use self::MultiStoreError::*;

#[derive(Debug)]
pub enum MultiStoreError<E> {
	NoSplit,
    StoreNotFound(String),
    WrappedError(E)
}

pub struct MultiStore<'a, T> {
    stores: HashMap<String, Box<AssetStore<T> + 'a>>
}

impl<'a, T> MultiStore<'a, T> {
    pub fn new() -> MultiStore<'a, T> {
        MultiStore { stores: HashMap::new() }
    }

    pub fn add<S: AssetStore<T> + 'a>(&mut self, prefix: &str, store: S) {
        self.stores.insert(prefix.to_string(), Box::new(store));
    }

    fn get_store<'b>(&self, path: &'b str) ->
    Result<(&Box<AssetStore<T>>, &'b str), MultiStoreError<T>> {
        let split: Vec<&str> = path.splitn(1, ':').collect();
        if split.len() == 1 {
            return Err(NoSplit)
        }
        let (before, after) = (split[0], split[1]);
        match self.stores.get(&before.to_string()) {
            Some(x) => Ok((x, after)),
            None => Err(StoreNotFound(before.to_string()))
        }
    }
}

// impl <'a> AssetStore for MultiStore<'a> {
//     fn load(&self, path: &str) {
//         match self.get_store(path) {
//             Ok((store, path)) => store.load(path),
//             Err(_) => {}
//         }
//     }

//     fn load_all<'b, I: Iterator<Item=&'b str>>(&self, paths: I) {
//         let mut paths = paths;
//         for path in paths {
//             match self.get_store(path) {
//                 Ok((store, path)) => store.load(path),
//                 Err(_) => {}
//             }
//         }
//     }

//     fn is_loaded(&self, path: &str) -> Result<bool, AssetStoreError>  {
//         let (store, path) = try!(self.get_store(path));
//         store.is_loaded(path).map_err(|e| WrappedError(e))
//     }

//     fn all_loaded<'b, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<bool, Vec<(&'b str, AssetStoreError)>>
//     where Self: Sized {
//         let mut paths = paths;
//         let mut errs = Vec::new();
//         let mut loaded = true;
//         for path in paths {
//             match self.get_store(path) {
//                 Ok((store, spath)) =>  {
//                     match store.is_loaded(spath) {
//                         Ok(b) => {
//                             loaded &= b;
//                         }
//                         Err(x) => {
//                             errs.push((path, WrappedError(x)));
//                         }
//                     }
//                 }
//                 Err(x) => errs.push((path, x))
//             }
//         }
//         if errs.len() > 0 {
//             Err(errs)
//         } else {
//             Ok(loaded)
//         }
//     }

//     fn unload(&self, path: &str) {
//         match self.get_store(path) {
//             Ok((store, path)) => store.unload(path),
//             Err(_) => {}
//         }
//     }

//     fn unload_all<'b, I: Iterator<Item=&'b str>>(&self, paths: I) {
//         let mut paths = paths;
//         for path in paths {
//             match self.get_store(path) {
//                 Ok((store, path)) => store.unload(path),
//                 Err(_) => {}
//             }
//         }
//     }

//     fn unload_everything(&self) {
//         for (_, store) in self.stores.iter() {
//             store.unload_everything();
//         }
//     }

//     fn map_resource<O, F>(&self , path: &str, mapfn: F) -> Result<Option<O>, AssetStoreError>
//         where F: Fn(&[u8]) -> O, Self: Sized {

//         let (store, path) = try!(self.get_store(path));
//         let mut ret = None;
//         let out = store.with_bytes(path, |bytes| {
//             ret = Some(mapfn(bytes));
//         });
//         match (out, ret) {
//             (Ok(Some(_)), Some(x)) => Ok(Some(x)),
//             (Ok(None), _) => Ok(None),
//             (Err(e), _) => Err(WrappedError(e)),
//             (Ok(Some(())), None) => unreachable!()
//         }
//     }

//     fn map_resource_block<O, F>(&self , path: &str, mapfn: F) -> Result<O, AssetStoreError>
//         where F: Fn(&[u8]) -> O, Self: Sized {

//         let (store, path) = try!(self.get_store(path));
//         let mut ret = None;
//         let out = store.with_bytes_block(path, |bytes| {
//             ret = Some(mapfn(bytes));
//         });
//         match (out, ret) {
//             (_, Some(x)) => Ok(x),
//             (Err(e), _) => Err(WrappedError(e)),
//             (Ok(_), None) => unreachable!()
//         }
//     }
// }
