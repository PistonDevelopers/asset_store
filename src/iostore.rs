use std::collections::HashMap;
use std::io::{IoError, OtherIoError, PermissionDenied, IoResult, File};
use std::io::timer::sleep;
use std::time::duration::Duration;
use std::sync::{Arc, RWLock};
use std::thread::Thread;

use hyper::Url;
use hyper::client::Response;
use hyper::Client;
use hyper::status::StatusCode;

use super::AssetStore;

type DistMap = Arc<RWLock<HashMap<String, IoResult<Vec<u8>>>>>;
pub trait IoBackend {
    fn go_get(&self, path: &str, mem: DistMap);
}

pub struct IoStore<Backend> {
    backend: Backend,
    mem: DistMap,
    //awaiting: HashSet<String>
}

pub fn from_directory(path: &str) -> IoStore<FsBackend> {
    let path = Path::new(path);
    IoStore {
        backend: FsBackend { path: path },
        mem: Arc::new(RWLock::new(HashMap::new())),
        //awaiting: HashSet::new(),
    }
}

pub fn from_url(base: &str) -> IoStore<NetBackend> {
    IoStore {
        backend: NetBackend { base: base.to_string() },
        mem: Arc::new(RWLock::new(HashMap::new())),
        //awaiting: HashSet::new(),
    }
}

impl <B: IoBackend> AssetStore<IoError> for IoStore<B> {
    fn load(&self, path: &str) {
        //if !self.awaiting.contains_equiv(&path) {
            self.backend.go_get(path, self.mem.clone());
        //}
        //self.awaiting.insert(path.to_string());
    }

    fn is_loaded(&self, path: &str) -> Result<bool, IoError> {
        let mem = self.mem.read();
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

    fn map_resource<O>(&self, path: &str, mapfn: |&[u8]| -> O) ->
    IoResult<Option<O>> {
        let mem = self.mem.read();
        match mem.get(path) {
            Some(&Ok(ref v)) => Ok(Some((mapfn)(v.as_slice()))),
            Some(&Err(ref e)) => Err(e.clone()),
            None => Ok(None)
        }
    }

    fn map_resource_block<O>(&self, path: &str, mapfn: |&[u8]| -> O)
    -> IoResult<O> {
        self.load(path);
        loop {
            {
                return match self.map_resource(path, |x| mapfn(x)) {
                    Ok(Some(v)) => Ok(v),
                    Err(e) => Err(e),
                    Ok(None) => { continue; }
                }
            }
            sleep(Duration::milliseconds(0));
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
        (file, File::open(&base).read_to_end())
    }
}

impl IoBackend for FsBackend {
    fn go_get(&self, file: &str, mem: DistMap) {
        let path = self.path.clone();
        let file = file.to_string();
        Thread::spawn(move || {
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
        let url = match Url::parse(path.as_slice()) {
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
        Thread::spawn(move || {
            let mut res = match NetBackend::http_get(&path) {
                Ok(res) => res,
                Err(err) => {
                    let error = Err(IoError {
                        kind: OtherIoError,
                        desc: "Error fetching file over http",
                        detail: Some(format!("for file {}: {}", path, err))
                    });
                    let mut map = mem.write();
                    map.insert(file, error);
                    return;
                }
            };

            if res.status == StatusCode::Ok {
                let mut map = mem.write();
                map.insert(file, res.read_to_end());
            } else {
                let error = Err(IoError {
                        kind: OtherIoError,
                        desc: "Error fetching file over http",
                        detail: Some(format!("for file {}: {}", path, res.status))
                });
                let mut map = mem.write();
                map.insert(file, error);
            }
        }).detach();
    }
}
