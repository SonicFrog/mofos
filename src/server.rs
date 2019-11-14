use std::collections::HashMap;
use std::convert::{Into, TryFrom};
use std::fs;
use std::io::{Error, ErrorKind, SeekFrom};
use std::net::{SocketAddr, UdpSocket};
use std::os::unix::fs::FileExt;
use std::path::Path;

use super::proto::*;

const MTU: u64 = 1500;

enum ServerStatus {
    ShouldExit,
    ShouldContinue,
    BadRequest,
    IOError,
}

pub struct MofosServer {
    socket: UdpSocket,
    pending: HashMap<u64, MofosResponse>,
    files: HashMap<String, fs::File>,
    destination: String,
}

impl MofosServer {
    pub fn new(addr: SocketAddr, dir: &Path) -> Result<MofosServer, Error> {
        if !dir.is_dir() {
            return Err(Error::new(ErrorKind::Other, "is not a directory"));
        }

        Ok(MofosServer {
            socket: UdpSocket::bind(addr)?,
            files: HashMap::new(),
            pending: HashMap::new(),
            destination: String::from(dir.to_str().unwrap()),
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        // TODO: chroot server into destination directory

        self.server_loop()
    }

    fn server_loop(&mut self) -> Result<(), Error> {
        // we should not send more than the MTU so 1500 is ok
        let buf: &mut [u8] = &mut [0u8; 1500];

        loop {
            match self.socket.recv_from(buf) {
                Ok((recvd, addr)) => {
                    match MofosRequest::try_from(&buf[0..recvd]) {
                        Ok(req) => match self.process_request(&req) {
                            Ok(resp) => {
                                let bytes: Vec<u8> = resp.into();

                                match self.socket.send_to(bytes.as_slice(), addr) {
                                    Ok(sent) => {
                                        println!("sent {} bytes", sent);
                                    }

                                    Err(e) => {
                                        return Err(e);
                                    }
                                };
                            }

                            Err(e) => {
                                println!("failed to process request: {}", e);
                            }
                        },

                        Err(e) => {
                            println!("invalid request received: {}", e);
                        }
                    };
                }

                Err(e) => {
                    println!("error reading from socket: {}", e);
                    return Err(e);
                }
            };
        }
    }

    fn process_request(&mut self, req: &MofosRequest) -> Result<MofosResponse, Error> {
        match req {
            MofosRequest::GetAttr { id, path } => match fs::metadata(path) {
                Ok(metadata) => {
                    let resp = MofosResponse::new_get_attr(*id, FileAttr::from(&metadata));

                    Ok(resp)
                }

                Err(e) => Err(e),
            },

            MofosRequest::Open { id, path, flags } => {
                if let Some(_) = self.files.get(path) {
                    return Ok(MofosResponse::new_open(*id, Status::Ok));
                }

                // TODO: handle flags
                match fs::File::open(path) {
                    Ok(file) => {
                        self.files.insert(path.to_string(), file);
                        Ok(MofosResponse::new_open(*id, Status::Ok))
                    }

                    Err(e) => Err(e),
                }
            }

            MofosRequest::Readdir { id, path, offset } => {
                // FIXME: dynamic entry count
                let entry_count: usize = 20;
                let native_entries: Vec<fs::DirEntry> = fs::read_dir(path)?
                    .take(entry_count)
                    .map(|e| e.unwrap())
                    .collect();
                let entries = native_entries.iter().map(|e| Entry::from(e)).collect();

                Ok(MofosResponse::new_readdir(*id, Status::Ok, entries))
            }

            MofosRequest::Write {
                id,
                path,
                data,
                offset,
            } => {
                if let Some(file) = self.files.get_mut(path) {
                    if let Err(e) = file.write_all_at(data, *offset as u64) {
                        unimplemented!("error writing file {}", e)
                    } else {
                        unimplemented!()
                    }
                } else {
                    Err(Error::new(ErrorKind::InvalidInput, "Unopened path"))
                }
            }

            MofosRequest::SetAttr { id, path, attrs } => {
                unimplemented!("setattr for {} at {} with {:?}", id, path, attrs)
            }

            MofosRequest::Read {
                id,
                path,
                size,
                offset,
            } => {
                if let Some(file) = self.files.get(path) {
                    let buf: &mut [u8] = &mut [0u8; 1500];
                    let read = file.read_at(&mut buf[0..*size as usize], *offset as u64)?;

                    Ok(MofosResponse::new_read(
                        *id,
                        Status::Ok,
                        Vec::from(&mut buf[0..read]),
                    ))
                } else {
                    Err(Error::new(ErrorKind::InvalidInput, "unopened path"))
                }
            }

            // TODO: handle other request types
            _ => Err(Error::new(
                ErrorKind::Other,
                "process_request: unimplemented",
            )),
        }
    }
}

#[cfg(test)]
mod test {
    extern crate mktemp;

    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    use self::mktemp::Temp;
    use super::MofosServer;

    const ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));

    fn setup_test() -> (MofosServer, Temp) {
        let temp = Temp::new_dir().expect("could not create temp dir");
        let srv = MofosServer::new(ADDR, &temp.to_path_buf()).expect("unable to start server");

        (srv, temp)
    }

    #[test]
    fn server_bind_test() {
        let (srv, tmp) = setup_test();

        assert!(tmp.to_path_buf().exists());
    }

    #[test]
    fn server_finds_file_test() {
        let (srv, tmp) = setup_test();
        let path = tmp.to_path_buf();

        // TODO: create file and access it through server
    }
}
