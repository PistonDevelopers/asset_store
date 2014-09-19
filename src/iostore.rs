use std::collections::hashmap::HashMap;
use std::io::IoError;
use std::io::PermissionDenied;
use std::io::IoResult;
use std::io::File;

use std::comm::Empty;
use std::comm::Disconnected;

use super::AssetStore;

pub struct IoStore {
    mem: HashMap<String, IoResult<Vec<u8>>>,
    incoming: Receiver<(String, IoResult<Vec<u8>>)>,
    outgoing: Sender<String>,
    disconnected: bool
}

impl IoStore {
    pub fn from_directory(path: Path) -> IoStore {
        let (snd_data, rec_data) = channel();
        let (snd_req, rec_req) = channel();

        let fs_worker = FsWorker {
            path: path,
            dump: snd_data,
            requests: rec_req
        };

        fs_worker.spawn();

        IoStore {
            mem: HashMap::new(),
            incoming: rec_data,
            outgoing: snd_req,
            disconnected: false
        }
    }

    fn update(&mut self) {
        loop {
            let upd = match self.incoming.try_recv() {
                Ok(value) => value,
                Err(Empty) => { break; }
                Err(Disconnected) => {
                    self.disconnected = true;
                    break;
                }
            };
            let (path, result) = upd;
            self.mem.insert(path, result);
        }
    }
}

impl AssetStore<IoError> for IoStore {
    fn load(&mut self, path: &str) {
        self.update();
        self.outgoing.send(path.to_string());
    }

    fn is_loaded(&mut self, path: &str) -> Result<bool, IoError> {
        self.update();
        match self.mem.find_equiv(&path) {
            Some(&Ok(_)) => Ok(true),
            Some(&Err(ref e)) => Err(e.clone()),
            None => Ok(false)
        }
    }

    fn unload(&mut self, path: &str) {
        self.update();
        self.mem.pop_equiv(&path);
    }

    fn unload_everything(&mut self) {
        self.update();
        self.mem.clear();
    }

    fn fetch<'s>(&'s mut self, path: &str) -> Result<Option<&'s Vec<u8>>, IoError> {
        self.update();
        match self.mem.find_equiv(&path) {
            Some(&Ok(ref v)) => Ok(Some(v)),
            Some(&Err(ref e)) => Err(e.clone()),
            None => Ok(None)
        }
    }

    fn fetch_block<'a>(&'a mut self, path: &str) -> Result<&'a Vec<u8>, IoError> {
        if self.mem.contains_key_equiv(&path) {
            match self.fetch(path) {
                Ok(Some(x)) => Ok(x),
                Err(e) => Err(e),
                Ok(None) => fail!()
            }
        } else {
            for obj in self.incoming.iter() {
                let (g_path, result) = obj;
                self.mem.insert(g_path.clone(), result);
                if path == g_path.as_slice() {
                    match self.mem.find_equiv(&path) {
                        Some(&Ok(ref x)) => return  Ok(x) ,
                        Some(&Err(ref e)) => return Err(e.clone()) ,
                        None => fail!()
                    }
                }
            }
            fail!()
        }
    }
}

struct FsWorker {
    path: Path,
    dump: Sender<(String, IoResult<Vec<u8>>)>,
    requests: Receiver<String>
}

impl FsWorker {
    fn spawn(self) {
        spawn(proc(){
            for incoming in self.requests.iter() {
                self.dump.send(self.process(incoming));
            }
        });
    }

    fn process(&self, file: String) -> (String, IoResult<Vec<u8>>) {
        let mut base = self.path.clone();
        base.push(file.clone());

        if !self.path.is_ancestor_of(&base) {
            let detail = format!("{} is not a child of {}",
                                 base.display(), self.path.display());
            return (
                file,
                Err(
                    IoError {
                        kind: PermissionDenied,
                        desc: "Attempt to escape filestore sandbox",
                        detail: Some(detail)
                    }
                )
            );
        }
        (file, File::open(&base).read_to_end())
    }
}
