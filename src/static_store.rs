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
    fn load(&mut self, _: &str) { }

    fn is_loaded(&mut self, path: &str) -> Result<bool, StaticStoreError> {
        Ok(self.find(path).is_some())
    }

    fn unload(&mut self, _: &str) { }

    fn unload_everything(&mut self) { }

    fn fetch<'s>(&'s mut self, path: &str) -> Result<Option<&'s [u8]>,
StaticStoreError> {
        match self.find(path) {
            Some(x) => Ok(Some(x)),
            None => Err(NotFound(path.to_string()))
        }
    }

    fn fetch_block<'s>(&'s mut self, path: &str) -> Result<&'s [u8], StaticStoreError> {
        match self.find(path) {
            Some(x) => Ok(x),
            None => Err(NotFound(path.to_string()))
        }
    }
}
