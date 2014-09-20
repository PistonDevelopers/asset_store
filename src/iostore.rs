use std::collections::hashmap::HashMap;
use std::collections::hashmap::HashSet;
use std::io::IoError;
use std::io::OtherIoError;
use std::io::PermissionDenied;
use std::io::IoResult;
use std::io::File;

use std::comm::Empty;
use std::comm::Disconnected;

use curl::http;

use super::AssetStore;

pub trait IoBackend {
    fn go_get(&self, path: &str, how_return: Sender<(String, IoResult<Vec<u8>>)>);
}

pub struct IoStore<Backend> {
    backend: Backend,
    mem: HashMap<String, IoResult<Vec<u8>>>,
    incoming: Receiver<(String, IoResult<Vec<u8>>)>,
    sender: Sender<(String, IoResult<Vec<u8>>)>,
    awaiting: HashSet<String>
}

pub fn from_directory(path: Path) -> IoStore<FsBackend> {
    let (sx, rx) = channel();
    IoStore {
        backend: FsBackend { path: path },
        mem: HashMap::new(),
        awaiting: HashSet::new(),
        incoming: rx,
        sender: sx
    }
}

pub fn from_url(beginning: String) -> IoStore<NetBackend> {
    let (sx, rx) = channel();
    IoStore {
        backend: NetBackend { beginning: beginning },
        mem: HashMap::new(),
        awaiting: HashSet::new(),
        incoming: rx,
        sender: sx
    }
}

impl<B: IoBackend> IoStore<B> {
    fn update(&mut self) {
        loop {
            let upd = match self.incoming.try_recv() {
                Ok(value) => value,
                Err(Empty) => { break; }
                Err(Disconnected) => {
                    break;
                }
            };
            let (path, result) = upd;
            self.mem.insert(path, result);
        }
    }
}

impl <B: IoBackend> AssetStore<IoError> for IoStore<B> {
    fn load(&mut self, path: &str) {
        self.update();
        // Don't load something that we are currently waiting on.
        if !self.awaiting.contains_equiv(&path) {
            self.backend.go_get(path, self.sender.clone());
        }
        self.awaiting.insert(path.to_string());
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
        self.load(path);
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

pub struct FsBackend {
    path: Path,
}

impl FsBackend {
    fn process(path: Path, file: String) -> (String, IoResult<Vec<u8>>) {
        let mut base = path.clone();
        base.push(file.clone());

        if !path.is_ancestor_of(&base) {
            let detail = format!("{} is not a child of {}",
                                 base.display(), path.display());
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
        println!("here");
        (file, File::open(&base).read_to_end())
    }
}

impl IoBackend for FsBackend {
    fn go_get(&self, file: &str, ret: Sender<(String, IoResult<Vec<u8>>)>) {
        let path = self.path.clone();
        let file = file.to_string();
        spawn(proc() {
            let _ = ret.send_opt(FsBackend::process(path, file));
        });
    }
}

pub struct NetBackend {
    beginning: String
}

impl IoBackend for NetBackend {
    fn go_get(&self, file: &str, ret: Sender<(String, IoResult<Vec<u8>>)>) {
        let mut path = self.beginning.clone();
        path.push_str(file);
        let file = file.to_string();
        spawn(proc() {
            let resp = match http::handle().get(path.as_slice()).exec() {
                Ok(b) => b.move_body(),
                Err(err) => {
                    let _ = ret.send_opt((file, Err(IoError {
                        kind: OtherIoError,
                        desc: "Error fetching file over http",
                        detail: Some(format!("for file {}: {}", path, err))
                    })));
                    return;
                }
            };

            let _ = ret.send_opt((file, Ok(resp)));
        });
    }
}
