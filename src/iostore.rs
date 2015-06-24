use std::path::{Path, PathBuf};
use std::fs::File;
use std::collections::HashMap;
use std::thread::{sleep_ms, spawn};
use std::time::duration::Duration;
use std::sync::{Arc, RwLock};
use std::thread::Thread;
use std::io;

use hyper::Url;
use hyper::client::Response;
use hyper::Client;
use hyper::status::StatusCode;

use super::AssetStore;

type DistMap = Arc<RwLock<HashMap<String, io::Result<Vec<u8>>>>>;
pub trait IoBackend {
    fn go_get(&self, path: &str, mem: DistMap);
}

pub struct IoStore<Backend> {
    backend: Backend,
    mem: DistMap,
    //awaiting: HashSet<String>
}

pub fn from_directory<P: Into<PathBuf>>(path: P) -> IoStore<FsBackend> {
    let path: PathBuf = path.into();
    IoStore {
        backend: FsBackend { path: path },
        mem: Arc::new(RwLock::new(HashMap::new())),
        //awaiting: HashSet::new(),
    }
}

pub fn from_url(base: &str) -> IoStore<NetBackend> {
    IoStore {
        backend: NetBackend { base: base.to_string() },
        mem: Arc::new(RwLock::new(HashMap::new())),
        //awaiting: HashSet::new(),
    }
}

impl <B: IoBackend> AssetStore<io::Error> for IoStore<B> {
    fn load(&self, path: &str) {
        //if !self.awaiting.contains_equiv(&path) {
            self.backend.go_get(path, self.mem.clone());
        //}
        //self.awaiting.insert(path.to_string());
    }

    fn is_loaded(&self, path: &str) -> Result<bool, io::Error> {
        let mem = match self.mem.read() {
                Ok(mem) => { mem },
                Err(_) => { return Err(io::Error::new(io::ErrorKind::Other, "Poisoned")); }
            };
        match mem.get(path) {
            Some(&Ok(_)) => Ok(true),
            Some(&Err(ref e)) => Err(e.clone()),
            None => Ok(false)
        }
    }

    fn unload(&self, path: &str) {
        let mut mem = self.mem.write();
        mem.remove(path);
    }

    fn unload_everything(&self) {
        let mut mem = self.mem.write();
        mem.clear();
    }

    fn map_resource<F, O>(&self, path: &str, mapfn: F)
    -> io::Result<Option<O>>
        where F: FnOnce(&[u8]) -> O
    {
        let mem = self.mem.read();
        match mem.get(path) {
            Some(&Ok(ref v)) => Ok(Some((mapfn)(&v))),
            Some(&Err(ref e)) => Err(e.clone()),
            None => Ok(None)
        }
    }

    fn map_resource_block<F, O>(&self, path: &str, mapfn: F)
    -> io::Result<O>
        where F: FnOnce(&[u8]) -> O
    {
        self.load(path);
        loop {
            {
                return match self.map_resource(path, |x| mapfn(x)) {
                    Ok(Some(v)) => Ok(v),
                    Err(e) => Err(e),
                    Ok(None) => { continue; }
                }
            }
            sleep_ms(0);
        }
    }
}

pub struct FsBackend {
    path: PathBuf,
}

impl FsBackend {
    fn process<P: AsRef<Path>>(path: P, filen: String) -> (String, io::Result<Vec<u8>>) {
        use std::fs::PathExt;
        use std::io::Read;

        let mut base = path.as_ref().to_path_buf();
        base.push(filen.clone());

        // is the path valid?
        if !base.exists() {
            return (
                filen.clone(),
                Err(
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Given path does not exist: {} does not contain {}", match path.as_ref().to_str() {
                            Some(s) => { s },
                            None => { "{Bad Path}"}
                        }, filen)
                    )
                )
            );
        }

        let mut file = File::open(&base);
        match file {
            Ok(mut f) => {
                let mut buf: Vec<u8> = Vec::new();
                match f.read_to_end(&mut buf) {
                    Ok(_) => { (filen, Ok(buf)) }
                    Err(e) => { (filen, Err(e)) }
                }
            },
            Err(e) => { (filen, Err(e)) }
        }
    }
}

impl IoBackend for FsBackend {
    fn go_get(&self, file: &str, mem: DistMap) {
        let path = self.path.clone();
        let file = file.to_string();
        spawn(move || {
            let (file, bytes) = FsBackend::process(path, file);
            let mut mem = mem.write();
            mem.insert(file, bytes);
        }).detach();
    }
}

pub struct NetBackend {
    base: String
}

impl NetBackend {
    fn http_get(path: &String) -> Result<Response, String> {
        let url = match Url::parse(path) {
            Ok(url) => url,
            Err(parse_err) => return Err(
                format!("Error parsing url: {}", parse_err)
            ),
        };

        let mut client = Client::new();
        let request = client.get(url);

        request.send().map_err(|e| e.to_string())
    }
}

impl IoBackend for NetBackend {
    fn go_get(&self, file: &str, mem: DistMap) {
        let path = vec![self.base.clone(), file.to_string()].concat();
        let file = file.to_string();
        spawn(move || {
            let mut res = match NetBackend::http_get(&path) {
                Ok(res) => res,
                Err(err) => {
                    let error = Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Error fetching file over http {}: {}", path, err)
                    ));
                    let mut map = mem.write();
                    map.insert(file, error);
                    return;
                }
            };

            if res.status == StatusCode::Ok {
                let mut map = mem.write();
                map.insert(file, res.read_to_end());
            } else {
                let error = Err(io::Error {
                        kind: io::ErrorKind::Other,
                        desc: "Error fetching file over http",
                        detail: Some(format!("for file {}: {}", path, res.status))
                });
                let mut map = mem.write();
                map.insert(file, error);
            }
        }).detach();
    }
}
