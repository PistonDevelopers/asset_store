use resources_package_package::Package;
use super::AssetStore;

use std::path::Path;

#[derive(Debug)]
pub enum StaticStoreError {
    NotFound(String)
}

pub struct StaticStore {
    mem: &'static Package,
}

impl StaticStore {
    pub fn new(m: &'static Package) -> StaticStore {
        StaticStore{ mem: m }
    }

    fn find(&self, path: &str) -> Option<&[u8]> {
        self.mem.find(&Path::new(path))
    }
}

impl AssetStore<StaticStoreError> for StaticStore {
    fn load(&self, _: &str) { }

    fn is_loaded(&self, path: &str) -> Result<bool, StaticStoreError> {
        Ok(self.find(path).is_some())
    }

    fn unload(&self, _: &str) { }

    fn unload_everything(&self) { }

    fn map_resource<O, F>(&self , path: &str, mapfn: F) -> Result<Option<O>, StaticStoreError>
        where F : Fn(&[u8]) -> O {

        match self.find(path) {
            Some(x) => Ok(Some(mapfn(x))),
            None => Err(StaticStoreError::NotFound(path.to_string()))
        }
    }

    fn map_resource_block<O, F>(&self, path: &str, mapfn: F) -> Result<O, StaticStoreError>
        where F : Fn(&[u8]) -> O {

        match self.map_resource(path, mapfn) {
            Ok(Some(x)) => Ok(x),
            Ok(None) => unreachable!(),
            Err(x) => Err(x)
        }
    }
}