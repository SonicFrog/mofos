extern crate fuse;
extern crate libc;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::net::SocketAddr;

use self::fuse::*;
use self::libc::c_int;

enum MofosData {
    Dir(String, u16),
    File(String, u16),
}

impl MofosData {
    fn new_file(path: String, perms: u16) -> MofosData {
        MofosData::File(path, perms)
    }

    fn new_dir(path: String, perms: u16) -> MofosData {
        MofosData::Dir(path, perms)
    }

    fn path(&self) -> &String {
        match self {
            MofosData::File(path, _) => path,
            MofosData::Dir(path, _) => path,
        }
    }

    fn perms(&self) -> u16 {
        match self {
            MofosData::File(_, perms) => *perms,
            MofosData::Dir(_, perms) => *perms,
        }
    }
}

pub struct MofosFS {
    addr: SocketAddr,
    last_ino: u64,
    inomap: HashMap<u64, MofosData>,
    fhs: HashMap<u64, MofosData>,
}

impl MofosFS {
    pub fn new(addr: SocketAddr, dir: &str) -> MofosFS {
        MofosFS {
            addr: addr,
            inomap: HashMap::new(),
            fhs: HashMap::new(),
            last_ino: 1,
        }
    }

    fn path_from_ino(&self, ino: u64) -> &String {
        if let Some(entry) = self.inomap.get(&ino) {
            match entry {
                MofosData::File(path, perms) => &path,
                MofosData::Dir(path, perms) => &path,
            }
        } else {
            panic!("invalid ino");
        }
    }
}

impl Filesystem for MofosFS {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        info!("initializing fuse...");

        self.inomap
            .insert(self.last_ino, MofosData::new_dir(String::from("/"), 0755));

        Ok(())
    }

 fn lookup(&mut self, _req: &Request, parent: u64,
              name: &OsStr, reply: ReplyEntry)
    {
        let s = name.to_str().unwrap();

        info!("lookup {}", s);
    }

    fn readlink(&mut self, _req: &Request, _ino: u64, reply: ReplyData) {
        error!("readlink: unimplemented");
    }

    fn mknod(&mut self, _req: &Request, _parent: u64, _name: &OsStr,
             _mode: u32, _rdev: u32, _reply: ReplyEntry) {
        panic!("mknod");
    }

    fn mkdir(&mut self,
             _req: &Request,
             _parent: u64,
             _name: &OsStr,
             _mode: u32,
             _reply: ReplyEntry) {
        panic!("mkdir");
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        info!("getattr for {}", ino);

        let path = self.path_from_ino(ino);

        info!("found path {} for ino {}", path, ino);
    }

    fn opendir(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        info!("opening dir {}", ino);

        reply.opened(0, flags)
    }

    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64,
               offset: i64, reply: ReplyDirectory) {
        info!("reading directory {}", ino);
    }
}
