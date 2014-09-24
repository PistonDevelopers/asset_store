use std::collections::hashmap::HashMap;

use super::AssetStore;

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
    fn load(&mut self, path: &str) {
        self.store.load(path);
    }

    fn load_all<'a, I: Iterator<&'a str>>(&mut self, paths: I) {
        self.store.load_all(paths);
    }

    fn is_loaded(&mut self, path: &str) -> Result<bool, T> {
        self.store.is_loaded(path).map_err(|e| (self.trans)(e))
    }

    fn all_loaded<'a, I: Iterator<&'a str>>(&mut self, paths: I) -> Result<bool, Vec<(&'a str, T)>> {
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

    fn unload(&mut self, path: &str) {
        self.store.unload(path);
    }

    fn unload_all<'a, I: Iterator<&'a str>>(&mut self, paths: I) {
        self.store.unload_all(paths);
    }

    fn unload_everything(&mut self) {
        self.store.unload_everything();
    }

    fn fetch(&mut self, path: &str) -> Result<Option<&[u8]>, T> {
        match self.store.fetch(path) {
            Ok(v) => Ok(v),
            Err(e) => Err((self.trans)(e))
        }
    }

    fn fetch_block(&mut self, path: &str) -> Result<&[u8], T> {
        match self.store.fetch_block(path) {
            Ok(v) => Ok(v),
            Err(e) => Err((self.trans)(e))
        }
    }
}

pub struct MultiStore<T> {
    stores: HashMap<String, Box<AssetStore<T> + 'static>>
}

impl <T> MultiStore<T> {
    pub fn new() -> MultiStore<T> {
        MultiStore { stores: HashMap::new() }
    }

    pub fn add<E, S: AssetStore<E>>(&mut self, prefix: &str, store: S, tr: fn(E) -> T) {
        let wrapped = StoreWrapper::new(store, tr);
        self.stores.insert(prefix.to_string(), box wrapped);
    }

    fn get_store<'a>(&mut self, path: &'a str) ->
    Result<(&mut Box<AssetStore<T> + 'static>, &'a str), MultiStoreError<T>> {
        let split: Vec<&str> = path.splitn(1, ':').collect();
        if split.len() == 1 {
            return Err(NoSplit)
        }
        let (before, after) = (split[0], split[1]);
        match self.stores.find_mut(&before.to_string()) {
            Some(x) => Ok((x, after)),
            None => Err(StoreNotFound(before.to_string()))
        }
    }
}

impl <T> AssetStore<MultiStoreError<T>> for MultiStore<T> {
    /// Tell the asset store to begin loading a resource.
    fn load(&mut self, path: &str) {
        match self.get_store(path) {
            Ok((store, path)) => store.load(path),
            Err(_) => {}
        }
    }
    /// Tell the asset store to begin loading all resources.
    fn load_all<'a, I: Iterator<&'a str>>(&mut self, paths: I) {
        let mut paths = paths;
        for path in paths {
            match self.get_store(path) {
                Ok((store, path)) => store.load(path),
                Err(_) => {}
            }
        }
    }

    /// Check to see if a resource has been loaded or not.
    fn is_loaded(&mut self, path: &str) -> Result<bool, MultiStoreError<T>>  {
        let (store, path) = try!(self.get_store(path));
        store.is_loaded(path).map_err(|e| WrappedError(e))
    }
    /// Check to see if everything has been loaded.
    fn all_loaded<'a, I: Iterator<&'a str>>(&mut self, paths: I) -> Result<bool, Vec<(&'a str, MultiStoreError<T>)>> {
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

    /// Remove this resouce from this asset store if it is loaded.
    fn unload(&mut self, path: &str) {
        match self.get_store(path) {
            Ok((store, path)) => store.unload(path),
            Err(_) => {}
        }
    }
    /// Remove all these resouces from this asset store if they
    /// are loaded.
    fn unload_all<'a, I: Iterator<&'a str>>(&mut self, paths: I) {
        let mut paths = paths;
        for path in paths {
            match self.get_store(path) {
                Ok((store, path)) => store.unload(path),
                Err(_) => {}
            }
        }
    }
    /// Remove every resouce from this asset store
    fn unload_everything(&mut self) {
        for (_, store) in self.stores.iter_mut() {
            store.unload_everything();
        }
    }

    /// Try to fetch a resource.
    /// If the resource is fully loaded, returns Ok(Some(resource))
    /// If the resource has not been loaded, returns Ok(None)
    /// If the resource failed to load, returns Err(e)
    fn fetch<'a>(&'a mut self , path: &str) -> Result<Option<&'a [u8]>, MultiStoreError<T>> {
        let (store, path) = try!(self.get_store(path));
        store.fetch(path).map_err(|e| WrappedError(e))
    }

    /// Try to fetch a resource.  If the resource has not been loaded yet, block
    /// until it is loaded.
    fn fetch_block<'a>(&'a mut self, path: &str) -> Result<&'a [u8], MultiStoreError<T>> {
        let (store, path) = try!(self.get_store(path));
        store.fetch_block(path).map_err(|e| WrappedError(e))
    }
}
