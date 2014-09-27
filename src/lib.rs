extern crate curl;
extern crate resources_package_package;

pub use iostore::{
    IoStore,
    FsBackend,
    NetBackend,
    from_directory,
    from_url,
};

pub use multi_store::{
    MultiStore,
    MultiStoreError,
};
pub use static_store::{
    StaticStore,
    StaticStoreError
};


mod multi_store;
mod iostore;
mod static_store;

#[cfg(test)]
mod test;

pub trait AssetStore<E> {
    /// Tell the asset store to begin loading a resource.
    fn load(&self, path: &str);
    /// Tell the asset store to begin loading all resources.
    fn load_all<'a, I: Iterator<&'a str>>(&self, paths: I) {
        let mut paths = paths;
        for s in paths {
            self.load(s);
        }
    }

    /// Check to see if a resource has been loaded or not.
    fn is_loaded(&self, path: &str) -> Result<bool, E>;
    /// Check to see if everything has been loaded.
    fn all_loaded<'a, I: Iterator<&'a str>>(&self, paths: I) ->
    Result<bool, Vec<(&'a str, E)>> {
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
    fn unload(&self, path: &str);
    /// Remove all these resouces from this asset store if they
    /// are loaded.
    fn unload_all<'a, I: Iterator<&'a str>>(&self, paths: I) {
        let mut paths = paths;
        for p in paths {
            self.unload(p);
        }
    }
    /// Remove every resouce from this asset store
    fn unload_everything(&self);

    /// Given a path to a resource and a transformation function,
    /// returns the result of the transformation function applied
    /// to the bytes of the resource if that resource is loaded.
    ///
    /// Returns `Ok(value)` if the resource is loaded and where `value`
    /// is the result of the transformation.
    /// Returns Ok(None) if the resource is not yet loaded.
    /// Returns Err(e) if the resource failed to open with an error.
    fn map_resource<O>(&self , path: &str, mapfn: |&[u8]| -> O) ->
    Result<Option<O>, E>;

    /// See `map_resource`.  This function blocks on read, so the only
    /// possible return values are `Ok(value)`, or `Err(e)`.
    fn map_resource_block<O>(&self, path: &str, mapfn: |&[u8]| -> O) ->
    Result<O, E>;

    /// Similar to map_resource, the user provides a path and a
    /// funciton.  The function is run only if the file is loaded
    /// without error.  The return value of the provided function
    /// is ignored, and a status is returned in the format given by
    /// map_resource, but with the uint `()` value in place of
    /// a mapped value.
    fn with_bytes(&self, path:&str, with_fn: |&[u8]| -> ()) ->
    Result<Option<()>, E> {
        self.map_resource(path, with_fn)
    }

    /// The same as `with_bytes_block` but blocking.
    fn with_bytes_block(&self, path:&str, with_fn: |&[u8]| -> ()) ->
    Result<(), E> {
        self.map_resource_block(path, with_fn)
    }
}
