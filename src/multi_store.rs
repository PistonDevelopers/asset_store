use std::collections::hashmap::HashMap;

use super::AssetStore;

#[deriving(Show)]
pub enum MultiStoreError<E> {
    NoSplit,
    StoreNotFound(String),
    WrappedError(E)
}

struct StoreWrapper<B, E, T> {
    store: B,
    trans: fn(E) -> T
}

impl <E, T, B: AssetStore<E>> StoreWrapper<B, E, T> {
    fn new(st: B, tr: fn(E) -> T) -> StoreWrapper<B, E, T> {
        StoreWrapper { store: st, trans: tr }
    }
}

impl <E, T, B: AssetStore<E>> AssetStore<T> for StoreWrapper<B, E, T> {
    fn load(&self, path: &str) {
        self.store.load(path);
    }

    fn load_all<'a, I: Iterator<&'a str>>(&self, paths: I) {
        self.store.load_all(paths);
    }

    fn is_loaded(&self, path: &str) -> Result<bool, T> {
        self.store.is_loaded(path).map_err(|e| (self.trans)(e))
    }

    fn all_loaded<'a, I: Iterator<&'a str>>(&self, paths: I) -> Result<bool, Vec<(&'a str, T)>> {
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

    fn unload_all<'a, I: Iterator<&'a str>>(&self, paths: I) {
        self.store.unload_all(paths);
    }

    fn unload_everything(&self) {
        self.store.unload_everything();
    }

    fn map_resource<O>(&self, path: &str, mapfn: |&[u8]| -> O) -> Result<Option<O>, T> {
        self.store.map_resource(path, mapfn).map_err(|x| (self.trans)(x))
    }

    fn map_resource_block<O>(&self, path: &str, mapfn: |&[u8]| -> O) -> Result<O, T> {
        self.store.map_resource_block(path, mapfn).map_err(|x| (self.trans)(x))
    }
}

pub struct MultiStore<T> {
    stores: HashMap<String, Box<AssetStore<T> + 'static>>
}

impl<T: 'static> MultiStore<T> {
    pub fn new() -> MultiStore<T> {
        MultiStore { stores: HashMap::new() }
    }

    pub fn add<E, S: 'static + AssetStore<E>>(
        &mut self, 
        prefix: &str, 
        store: S, 
        tr: fn(E) -> T
    ) {
        let wrapped = StoreWrapper::new(store, tr);
        self.stores.insert(prefix.to_string(), box wrapped);
    }

    fn get_store<'a>(&self, path: &'a str) ->
    Result<(&Box<AssetStore<T> + 'static>, &'a str), MultiStoreError<T>> {
        let split: Vec<&str> = path.splitn(1, ':').collect();
        if split.len() == 1 {
            return Err(NoSplit)
        }
        let (before, after) = (split[0], split[1]);
        match self.stores.find(&before.to_string()) {
            Some(x) => Ok((x, after)),
            None => Err(StoreNotFound(before.to_string()))
        }
    }
}

impl<T: 'static> AssetStore<MultiStoreError<T>> for MultiStore<T> {
    fn load(&self, path: &str) {
        match self.get_store(path) {
            Ok((store, path)) => store.load(path),
            Err(_) => {}
        }
    }

    fn load_all<'a, I: Iterator<&'a str>>(&self, paths: I) {
        let mut paths = paths;
        for path in paths {
            match self.get_store(path) {
                Ok((store, path)) => store.load(path),
                Err(_) => {}
            }
        }
    }

    fn is_loaded(&self, path: &str) -> Result<bool, MultiStoreError<T>>  {
        let (store, path) = try!(self.get_store(path));
        store.is_loaded(path).map_err(|e| WrappedError(e))
    }

    fn all_loaded<'a, I: Iterator<&'a str>>(&self, paths: I) -> Result<bool, Vec<(&'a str, MultiStoreError<T>)>> {
        let mut paths = paths;
        let mut errs = Vec::new();
        let mut loaded = true;
        for path in paths {
            match self.get_store(path) {
                Ok((store, spath)) =>  {
                    match store.is_loaded(spath) {
                        Ok(b) => {
                            loaded &= b;
                        }
                        Err(x) => {
                            errs.push((path, WrappedError(x)));
                        }
                    }
                }
                Err(x) => errs.push((path, x))
            }
        }
        if errs.len() > 0 {
            Err(errs)
        } else {
            Ok(loaded)
        }
    }

    fn unload(&self, path: &str) {
        match self.get_store(path) {
            Ok((store, path)) => store.unload(path),
            Err(_) => {}
        }
    }

    fn unload_all<'a, I: Iterator<&'a str>>(&self, paths: I) {
        let mut paths = paths;
        for path in paths {
            match self.get_store(path) {
                Ok((store, path)) => store.unload(path),
                Err(_) => {}
            }
        }
    }

    fn unload_everything(&self) {
        for (_, store) in self.stores.iter() {
            store.unload_everything();
        }
    }

    fn map_resource<O>(&self , path: &str, mapfn: |&[u8]| -> O) ->
    Result<Option<O>, MultiStoreError<T>> {
        let (store, path) = try!(self.get_store(path));
        let mut ret = None;
        let out = store.with_bytes(path, |bytes| {
            ret = Some(mapfn(bytes));
        });
        match (out, ret) {
            (Ok(Some(_)), Some(x)) => Ok(Some(x)),
            (Ok(None), _) => Ok(None),
            (Err(e), _) => Err(WrappedError(e)),
            (Ok(Some(())), None) => unreachable!()
        }
    }

    fn map_resource_block<O>(&self, path: &str, mapfn: |&[u8]| -> O) ->
    Result<O, MultiStoreError<T>> {
        let (store, path) = try!(self.get_store(path));
        let mut ret = None;
        let out = store.with_bytes_block(path, |bytes| {
            ret = Some(mapfn(bytes));
        });
        match (out, ret) {
            (_, Some(x)) => Ok(x),
            (Err(e), _) => Err(WrappedError(e)),
            (Ok(_), None) => unreachable!()
        }
    }
}
