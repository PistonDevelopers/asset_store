use std::collections::HashMap;
use std::convert::From;
use std::error::Error;
use std::io::ErrorKind;
use std::fs::File;
#[allow(unused_imports)]
use std::path::{self, Path, PathBuf};
use std::string::String;
use std::sync::{Arc, RwLock};
#[allow(unused_imports)]
use std::thread::{self, spawn, sleep_ms};

use std::io::Error as IoError;
use std::io::Result as IoResult;

// use hyper::Url;
// use hyper::client::Response;
// use hyper::Client;
// use hyper::status::StatusCode;


use super::{AssetStore, AssetStoreError};

type DistMap = Arc<RwLock<HashMap<String, IoResult<Vec<u8>>>>>;
pub trait IoBackend {
    fn go_get(&self, path: &str, mem: DistMap);
}

pub struct IoStore<Backend> {
    backend: Backend,
    mem: DistMap,
    //awaiting: HashSet<String>
}

pub fn from_directory(path: &str) -> IoStore<FsBackend> {
    let path = PathBuf::from(String::from(path));
    IoStore {
        backend: FsBackend { path: path },
        mem: Arc::new(RwLock::new(HashMap::new())),
        //awaiting: HashSet::new(),
    }
}

impl <B: IoBackend> AssetStore for IoStore<B> {
    fn load(&self, path: &str) {
        //if !self.awaiting.contains_equiv(&path) {
            self.backend.go_get(path, self.mem.clone());
        //}
        //self.awaiting.insert(path.to_string());
    }

    fn is_loaded(&self, path: &str) -> Result<bool, AssetStoreError> {
        let mem = match self.mem.read() {
            Ok(mem) => { mem },
            Err(_) => { return Err(AssetStoreError::FileError(IoError::new(ErrorKind::Other, "Poisoned thread"))); }
        };

        match mem.get(path) {
            Some(&Ok(_)) => Ok(true),
            Some(&Err(ref e)) => Err(AssetStoreError::FileError(IoError::new(e.kind(), e.description().clone()))),
            None => Ok(false)
        }
        // Ok(true)
    }

    fn unload(&self, path: &str) {
        match self.mem.write() {
            Ok(mut mem) => { mem.remove(path); },
            Err(_) => { }
        }
    }

    fn unload_everything(&self) {
        match self.mem.write() {
            Ok(mut mem) => { mem.clear(); },
            Err(_) => { }
        }
    }

    fn map_resource<O, F>(&self , path: &str, mapfn: F) ->
    Result<Option<O>, AssetStoreError> where F: Fn(&[u8]) -> O {

        let mem = match self.mem.read() {
            Ok(mem) => { mem },
            Err(_) => { return Err(AssetStoreError::FileError(IoError::new(ErrorKind::Other, "Poisoned thread"))); }
        };

        match mem.get(path) {
            Some(&Ok(ref v)) => Ok(Some((mapfn)(&v[..]))),
            Some(&Err(ref e)) => Err(AssetStoreError::FileError(IoError::new(e.kind(), e.description().clone()))),
            None => Ok(None)
        }
    }

    fn map_resource_block<O, F>(&self , path: &str, mapfn: F) ->
    Result<O, AssetStoreError> where F: Fn(&[u8]) -> O {
        self.load(path);
        loop {
            {
                return match self.map_resource(path, |x| mapfn(x)) {
                    Ok(Some(v)) => Ok(v),
                    Err(e) => Err(e),
                    Ok(None) => { continue; }
                }
            }
            //sleep(Duration::milliseconds(0));
            sleep_ms(0);
        }
    }
}

pub struct FsBackend {
    path: PathBuf,
}

impl FsBackend {
    fn process<P: AsRef<Path>>(path: P, filen: String) -> (String, IoResult<Vec<u8>>) {
        use std::fs::PathExt;
        use std::io::Read;

        let mut base = path.as_ref().to_path_buf();
        base.push(filen.clone());

        // is the path valid?
        if !base.exists() {
            return (
                filen.clone(),
                Err(
                    IoError::new(
                        ErrorKind::NotFound,
                        format!("Given path does not exist: {} does not contain {}", match path.as_ref().to_str() {
                            Some(s) => { s },
                            None => { "{Bad Path}"}
                        }, filen)
                    )
                )
            );
        }

        match File::open(&base) {
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
        thread::spawn(move || {
            let (file, bytes) = FsBackend::process(path, file);
            if let Ok(mut mem) = mem.write() {
                mem.insert(file, bytes);
            };
        });
    }
}

// pub fn from_url(base: &str) -> IoStore<NetBackend> {
//     IoStore {
//         backend: NetBackend { base: base.to_string() },
//         mem: Arc::new(RwLock::new(HashMap::new())),
//         //awaiting: HashSet::new(),
//     }
// }

// // pub struct NetBackend {
// //     base: String
// // }

// // impl NetBackend {
// //     fn http_get(path: &String) -> Result<Response, String> {
// //         let url = match Url::parse(path.as_slice()) {
// //             Ok(url) => url,
// //             Err(parse_err) => return Err(
// //                 format!("Error parsing url: {}", parse_err)
// //             ),
// //         };

// //         let mut client = Client::new();
// //         let request = client.get(url);

// //         request.send().map_err(|e| e.to_string())
// //     }
// // }

// // impl IoBackend for NetBackend {
// //     fn go_get(&self, file: &str, mem: DistMap) {
// //         let path = vec![self.base.clone(), file.to_string()].concat();
// //         let file = file.to_string();
// //         Thread::spawn(move || {
// //             let mut res = match NetBackend::http_get(&path) {
// //                 Ok(res) => res,
// //                 Err(err) => {
// //                     let error = Err(IoError {
// //                         kind: OtherIoError,
// //                         desc: "Error fetching file over http",
// //                         detail: Some(format!("for file {}: {}", path, err))
// //                     });
// //                     let mut map = mem.write();
// //                     map.insert(file, error);
// //                     return;
// //                 }
// //             };

// //             if res.status == StatusCode::Ok {
// //                 let mut map = mem.write();
// //                 map.insert(file, res.read_to_end());
// //             } else {
// //                 let error = Err(IoError {
// //                         kind: OtherIoError,
// //                         desc: "Error fetching file over http",
// //                         detail: Some(format!("for file {}: {}", path, res.status))
// //                 });
// //                 let mut map = mem.write();
// //                 map.insert(file, error);
// //             }
// //         }).detach();
// //     }
// // }
