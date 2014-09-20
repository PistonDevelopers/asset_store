extern crate curl;

pub use iostore::{
    IoStore,
    FsBackend,
    NetBackend,
    from_directory,
    from_url
};
mod iostore;

#[cfg(test)]
mod test;

pub trait AssetStore<E> {
    /// Tell the asset store to begin loading a resource.
    fn load(&mut self, path: &str);
    /// Tell the asset store to begin loading all resources.
    fn load_all<'a, I: Iterator<&'a str>>(&mut self, paths: I) {
        let mut paths = paths;
        for s in paths {
            self.load(s);
        }
    }

    /// Check to see if a resource has been loaded or not.
    fn is_loaded(&mut self, path: &str) -> Result<bool, E>;
    /// Check to see if everything has been loaded.
    fn all_loaded<'a, I: Iterator<&'a str>>(&mut self, paths: I) -> Result<bool, Vec<(&'a str, E)>> {
        let mut paths = paths;
        let mut status = true;
        let mut errs = vec![];
        for p in paths {
            match self.is_loaded(p) {
                Ok(b) => {
                    status &= b;
                }
                Err(e) => {
                    errs.push((p, e));
                }
            }
        }
        if errs.len() == 0 {
            Ok(status)
        } else {
            Err(errs)
        }
    }

    /// Remove this resouce from this asset store if it is loaded.
    fn unload(&mut self, path: &str);
    /// Remove all these resouces from this asset store if they
    /// are loaded.
    fn unload_all<'a, I: Iterator<&'a str>>(&mut self, paths: I) {
        let mut paths = paths;
        for p in paths {
            self.unload(p);
        }
    }
    /// Remove every resouce from this asset store
    fn unload_everything(&mut self);

    /// Try to fetch a resource.
    /// If the resource is fully loaded, returns Ok(Some(resource))
    /// If the resource has not been loaded, returns Ok(None)
    /// If the resource failed to load, returns Err(e)
    fn fetch<'a>(&'a mut self , path: &str) -> Result<Option<&'a Vec<u8>>, E>;

    /// Try to fetch a resource.  If the resource has not been loaded yet, block
    /// until it is loaded.
    fn fetch_block<'a>(&'a mut self, path: &str) -> Result<&'a Vec<u8>, E>;
}

