use std::collections::HashMap;
use std::marker::PhantomData;

use super::AssetStore;
use self::MultiStoreError::*;

#[derive(Debug)]
pub enum MultiStoreError<E> {
    NoSplit,
    StoreNotFound(String),
    WrappedError(E)
}

struct StoreWrapper<S, E, T, F: Fn(E) -> T> {
    store: S,
    trans: F,
    _e: PhantomData<*const E>,
    _t: PhantomData<*const T>
}

impl <S, E, T, F: Fn(E) -> T> StoreWrapper<S, E, T, F> {
    fn new(st: S, tr: F) -> StoreWrapper<S, E, T, F> {
        StoreWrapper {
            store: st,
            trans: tr,
            _e: PhantomData,
            _t: PhantomData
        }
    }
}

impl <S: AssetStore<E>, E, T, F: Fn(E) -> T> AssetStore<T> for StoreWrapper<S, E, T, F> {
    fn load(&self, path: &str) {
        self.store.load(path);
    }

    fn load_all<'a, I: Iterator<Item=&'a str>>(&self, paths: I) {
        self.store.load_all(paths);
    }

    fn is_loaded(&self, path: &str) -> Result<bool, T> {
        self.store.is_loaded(path).map_err(|e| (self.trans)(e))
    }

    fn all_loaded<'a, I: Iterator<Item=&'a str>>(&self, paths: I) -> Result<bool, Vec<(&'a str, T)>> {
        let res = self.store.all_loaded(paths);
        match res {
            Ok(b) => Ok(b),
            Err(errs) =>
                Err(errs
                    .into_iter()
                    .map(|(n, e)| (n, (self.trans)(e)))
                    .collect())
        }
    }

    fn unload(&self, path: &str) {
        self.store.unload(path);
    }

    fn unload_all<'a, I: Iterator<Item=&'a str>>(&self, paths: I) {
        self.store.unload_all(paths);
    }

    fn unload_everything(&self) {
        self.store.unload_everything();
    }

    fn map_resource<O, M>(&self , path: &str, mapfn: M) -> Result<Option<O>, T>
        where M: Fn(&[u8]) -> O {

            self.store.map_resource(path, mapfn).map_err(|x| (self.trans)(x))
    }

    fn map_resource_block<O, M>(&self, path: &str, mapfn: M) -> Result<O, T>
        where M: Fn(&[u8]) -> O {

        self.store.map_resource_block(path, mapfn).map_err(|x| (self.trans)(x))
    }
}

pub struct MultiStore<'a, T> {
    stores: HashMap<String, Box<AssetStore<T> + 'a>>
}

impl<'a, T: 'a> MultiStore<'a, T> {
    pub fn new() -> MultiStore<'a, T> {
        MultiStore { stores: HashMap::new() }
    }

    pub fn add<E: 'a, S, F: 'a>(
        &mut self,
        prefix: &str,
        store: S,
        tr: F
    ) where S: 'a + AssetStore<E>, F: Fn(E) -> T {
        let wrapped = StoreWrapper::new(store, tr);
        self.stores.insert(prefix.to_string(), Box::new(wrapped));
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

// impl<'a, T: 'a> AssetStore<MultiStoreError<T>> for MultiStore<'a, T> {
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

//     fn is_loaded(&self, path: &str) -> Result<bool, MultiStoreError<T>>  {
//         let (store, path) = try!(self.get_store(path));
//         store.is_loaded(path).map_err(|e| WrappedError(e))
//     }

//     fn all_loaded<'b, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<bool, Vec<(&'b str, MultiStoreError<T>)>>
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

//     fn map_resource<O, F>(&self , path: &str, mapfn: F) -> Result<Option<O>, MultiStoreError<T>>
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

//     fn map_resource_block<O, F>(&self , path: &str, mapfn: F) -> Result<O, MultiStoreError<T>>
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

