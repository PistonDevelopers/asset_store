use super::AssetStore;

#[deriving(Show)]
pub enum StaticStoreError {
    NotFound(String)
}

pub struct StaticStore {
    mem: &'static [(&'static [u8], &'static [u8])]
}

impl StaticStore {
    pub fn new(m: &'static [(&'static [u8], &'static [u8])]) -> StaticStore {
        StaticStore{ mem: m }
    }

    fn find(&self, path: &str) -> Option<&[u8]> {
        let found =
            self.mem.iter()
                    .filter(|&&(s, _)| s == path.as_bytes())
                    .nth(0);
        match found {
            Some(&(_, bytes)) => Some(bytes),
            None => None
        }
    }
}

impl AssetStore<StaticStoreError> for StaticStore {
    fn load(&self, _: &str) { }

    fn is_loaded(&self, path: &str) -> Result<bool, StaticStoreError> {
        Ok(self.find(path).is_some())
    }

    fn unload(&self, _: &str) { }

    fn unload_everything(&self) { }

    fn map_resource<O>(&self , path: &str, mapfn: |&[u8]| -> O) -> Result<Option<O>, StaticStoreError> {
        match self.find(path) {
            Some(x) => Ok(Some(mapfn(x))),
            None => Err(NotFound(path.to_string()))
        }
    }

    fn map_resource_block<O>(&self, path: &str, mapfn: |&[u8]| -> O) -> Result<O, StaticStoreError> {
        match self.map_resource(path, mapfn) {
            Ok(Some(x)) => Ok(x),
            Ok(None) => unreachable!(),
            Err(x) => Err(x)
        }
    }
}
