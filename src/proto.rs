extern crate bincode;

use std::convert::{TryFrom, TryInto};
use std::fs::{DirEntry, FileType, Metadata};
use std::os::unix::fs::{DirEntryExt, FileTypeExt, MetadataExt};

use self::bincode::{deserialize, serialize, ErrorKind};

#[repr(u8)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Status {
    Ok = 0,
    NotFound = 1,
    Denied = 2,
    IOError = 3,

    Unknown = 0xff,
}

#[derive(Serialize, Deserialize)]
pub enum MofosRequest {
    GetAttr {
        id: u64,
        path: String,
    },
    SetAttr {
        id: u64,
        path: String,
        attrs: FileAttr,
    },

    Open {
        id: u64,
        path: String,
        flags: u32,
    },
    OpenDir {
        id: u64,
        path: String,
        flags: u32,
    },
    Readdir {
        id: u64,
        path: String,
        offset: i64,
    },

    MkNod {
        id: u64,
        path: String,
        flags: u32,
    },
    MkDir {
        id: u64,
        path: String,
        mode: u32,
    },

    Write {
        id: u64,
        path: String,
        data: Vec<u8>,
        offset: i64,
    },
    Read {
        id: u64,
        path: String,
        size: u32,
        offset: i64,
    },
    Unlink {
        id: u64,
        path: String,
    },

    Exit,
}

impl MofosRequest {
    pub fn new_open(id: u64, path: String, flags: u32) -> MofosRequest {
        MofosRequest::Open { id, path, flags }
    }

    pub fn new_readdir(id: u64, path: String, offset: i64) -> MofosRequest {
        MofosRequest::Readdir { id, path, offset }
    }
}

impl<'a> TryFrom<&'a [u8]> for MofosRequest {
    type Error = Box<ErrorKind>;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        deserialize(data)
    }
}

impl TryInto<Vec<u8>> for MofosRequest {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        serialize(&self)
    }
}

#[derive(Serialize, Deserialize)]
pub enum MofosResponse {
    GetAttr(u64, Status, FileAttr),
    SetAttr(u64, Status),

    Lookup(u64, Status, FileAttr),
    Open(u64, Status),

    Read(u64, Status, Vec<u8>),
    Readdir(u64, Status, Vec<Entry>),
}

impl MofosResponse {
    pub fn new_get_attr(id: u64, attrs: FileAttr) -> MofosResponse {
        MofosResponse::GetAttr(id, Status::Ok, attrs)
    }

    pub fn new_open(id: u64, status: Status) -> MofosResponse {
        MofosResponse::Open(id, status)
    }

    pub fn new_readdir(id: u64, status: Status, entries: Vec<Entry>) -> MofosResponse {
        MofosResponse::Readdir(id, status, entries)
    }

    pub fn new_read(id: u64, status: Status, data: Vec<u8>) -> MofosResponse {
        MofosResponse::Read(id, status, data)
    }
}

impl<'a> TryFrom<&'a [u8]> for MofosResponse {
    type Error = Box<ErrorKind>;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        deserialize(data)
    }
}

impl Into<Vec<u8>> for MofosResponse {
    fn into(self) -> Vec<u8> {
        serialize(&self).expect("tried to serialize invalid response")
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileAttr {
    ino: u64,
    tpe: Type,
    size: u64,
    uid: u32,
    gid: u32,
    mode: u16,
}

impl<'a> From<&'a DirEntry> for FileAttr {
    fn from(entry: &'a DirEntry) -> Self {
        let md = entry.metadata().expect("failed to read metadata");

        FileAttr::from(&md)
    }
}

impl<'a> From<&'a Metadata> for FileAttr {
    fn from(entry: &'a Metadata) -> Self {
        FileAttr {
            ino: entry.ino(),
            tpe: Type::from(entry.file_type()),
            size: entry.size(),
            uid: entry.uid(),
            gid: entry.gid(),
            mode: entry.mode() as u16,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Entry {
    name: String,
    attrs: FileAttr,
}

impl<'a> From<&'a DirEntry> for Entry {
    fn from(v: &'a DirEntry) -> Self {
        // TODO: handle errors
        Entry {
            name: String::from(v.path().to_str().unwrap()),
            attrs: FileAttr::from(&v.metadata().unwrap()),
        }
    }
}

#[repr(u8)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Type {
    File = 0,
    Dir = 1,
    Link = 2,
    Socket = 3,
    Fifo = 4,
    CharDev = 5,
    BlockDev = 6,

    Unknown = 0xff,
}

impl From<FileType> for Type {
    fn from(t: FileType) -> Self {
        if t.is_dir() {
            Type::Dir
        } else if t.is_file() {
            Type::File
        } else if t.is_symlink() {
            Type::Link
        } else if t.is_block_device() {
            Type::BlockDev
        } else if t.is_socket() {
            Type::Socket
        } else if t.is_char_device() {
            Type::CharDev
        } else if t.is_fifo() {
            Type::Fifo
        } else {
            Type::Unknown
        }
    }
}
