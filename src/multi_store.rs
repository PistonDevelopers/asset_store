// use std::collections::HashMap;

// use super::{AssetStore, AssetStoreError};
// use super::AssetStoreError::*;

// pub struct MultiStore<'a> {
//     stores: HashMap<String, Box<&'a AssetStore>>
// }

// impl<'a> MultiStore<'a> {
//     pub fn new() -> MultiStore {
//         MultiStore { stores: HashMap::new() }
//     }

//     pub fn add<E, S: AssetStore>(
//         &mut self,
//         prefix: &str,
//         store: S
//     ) {
//         self.stores.insert(prefix.to_string(), Box::new(store));
//     }

//     fn get_store<'a>(&self, path: &'a str) ->
//     Result<(&Box<AssetStore>, &'a str), AssetStoreError> {
//         let split: Vec<&str> = path.splitn(1, ':').collect();
//         if split.len() == 1 {
//             return Err(NoSplit)
//         }
//         let (before, after) = (split[0], split[1]);
//         match self.stores.get(&before.to_string()) {
//             Some(x) => Ok((x, after)),
//             None => Err(StoreNotFound(before.to_string()))
//         }
//     }
// }

// impl<T: 'static> AssetStore<MultiStoreError<T>> for MultiStore<T> {
//     fn load(&self, path: &str) {
//         match self.get_store(path) {
//             Ok((store, path)) => store.load(path),
//             Err(_) => {}
//         }
//     }

//     fn load_all<'a, I: Iterator<Item=&'a str>>(&self, paths: I) {
//         let mut paths = paths;
//         for path in paths {
//             match self.get_store(path) {
//                 Ok((store, path)) => store.load(path),
//                 Err(_) => {}
//             }
//         }
//     }

//     fn is_loaded(&self, path: &str) -> Result<bool, MultiStoreError<T>>  {
//         let (store, path) = try!(self.get_store(path));
//         store.is_loaded(path).map_err(|e| WrappedError(e))
//     }

//     fn all_loaded<'a, I: Iterator<Item=&'a str>>(&self, paths: I) -> Result<bool, Vec<(&'a str, MultiStoreError<T>)>> {
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

//     fn unload_all<'a, I: Iterator>(&self, paths: I) {
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

//     fn map_resource<O, F>(&self , path: &str, mapfn: F) -> Result<Option<O>, MultiStoreError<T>>
//         where F: Fn(&[u8]) -> O {

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

//     fn map_resource_block<O, F>(&self , path: &str, mapfn: F) -> Result<O, MultiStoreError<T>>
//         where F: Fn(&[u8]) -> O {

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
