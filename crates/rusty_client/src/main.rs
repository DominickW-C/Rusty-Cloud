use reqwest::blocking::Client;

use std::env;
use fuser::{
    Filesystem,
    Request,
    INodeNo,
    FileHandle,
    RenameFlags,
    OpenFlags,
    LockOwner,
    ReplyData,
    ReplyAttr,
    ReplyEmpty,
    ReplyOpen,
    ReplyCreate,
    ReplyWrite,
    ReplyDirectory,
    ReplyEntry,
    WriteFlags,
    MountOption,
    Config,
    SessionACL,
    BsdFileFlags,
    TimeOrNow,
};

use core::time;
use std::ffi::OsStr;
use std::path::Path;
use std::time::SystemTime;
use std::fs;

const TIMEOUT: time::Duration = time::Duration::new(2, 0);

struct RustyClient {
    http_client: Client
}

impl RustyClient{
    pub fn new() -> Self {
        let http_client = Client::new();
        Self { http_client }
    }
}

impl Filesystem for RustyClient {

    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
    }

    fn setattr(&self, _req: &Request, ino: INodeNo, _mode: Option<u32>, _uid: Option<u32>, _gid: Option<u32>, size: Option<u64>,
        _attime: Option<TimeOrNow>, _mtime: Option<TimeOrNow>, _ctime: Option<SystemTime>, _fh: Option<FileHandle>, 
        _crtime: Option<SystemTime>, _chgtime: Option<SystemTime>, _bkuptime: Option<SystemTime>, _flags: Option<BsdFileFlags>, reply: ReplyAttr) {
    }

    fn create(&self, _req: &Request, parent: INodeNo, name: &OsStr, _mode: u32, _umask: u32, _flags: i32, reply: ReplyCreate) {
    }

    fn mkdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, _mode: u32, _umask: u32, reply: ReplyEntry) {
    }

    fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
    }

    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
    }

    fn rename(&self, _req: &Request, parent: INodeNo, name: &OsStr, newparent: INodeNo, 
              newname: &OsStr, _flags: RenameFlags, reply: ReplyEmpty) {
    }
    fn open(&self, _req: &Request, _ino: INodeNo, _flags: OpenFlags, reply: ReplyOpen) {
    }

    fn read(&self, _req: &Request, ino: INodeNo, _fh: FileHandle, _offset: u64, _size: u32, 
        _flags: OpenFlags, _lock_owner: Option<LockOwner>, reply: ReplyData) {
    }

    fn readdir(&self, _req: &Request, ino: INodeNo, _fh: FileHandle, offset: u64, mut reply: ReplyDirectory) {
    }

    fn write(&self, _req: &Request, ino: INodeNo, _fh:FileHandle, _offset: u64, data: &[u8], 
        _write_flags: WriteFlags, _flags: OpenFlags, _lock_owner: Option<LockOwner>, reply: ReplyWrite) {
    }

    fn link(&self, _req: &Request, ino: INodeNo, newparent: INodeNo, newname: &OsStr, reply: ReplyEntry)  {
    }

    fn symlink (&self, _req: &Request, parent: INodeNo, link_name: &OsStr, target: &Path, reply: ReplyEntry) {
    }

    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
    }

    fn release(&self, _req: &Request, _ino: INodeNo, _fh: FileHandle, _flags: OpenFlags, _lock_owner: Option<LockOwner>,
        _flush: bool, reply: ReplyEmpty) {
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    //exe, host, location
    if args.len() < 3  || args.len() > 3 {
        println!("Missing needed parameters");
        println!("Usage: rustyfs [host] [mount location]");
        for ii in args.iter() {
            println!("{}", ii);
        }
        return
    }

    //valid amount of inputs, now check if host and location valid
    if args[2].starts_with('/') {
        println!("absolute path");
    } else {
        println!("relative path");
    }

    match fs::exists(&args[2]) {
        Ok(exists) => {
            if !exists {
                let _ = fs::create_dir_all(&args[2]);
                println!("created a new directory");
            } else {
                println!("directory already existed, using it");
            }
        },
        Err(_) => {
            println!("this should be impossible");
            return
        }
    }

    let mountpoint = Path::new(&args[2]);
    let mut options = Config::default();

    options.mount_options = vec![
        MountOption::AutoUnmount
    ];

    options.acl = SessionACL::All;

    let mounted_fs = fuser::mount(RustyClient::new(), mountpoint, &options);

    if let Err(e) = mounted_fs {
        println!("error mounting filesystem");
        println!("{}", e);
    }
}
