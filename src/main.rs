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
    ReplyWrite,
    ReplyEntry,
    WriteFlags
};

use std::ffi::OsStr;

struct RustyCloud;
const MNT_POINT: u64 = 1;
const TEST_FILE: u64 = 2;


impl Filesystem for RustyCloud {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        //TODO: Implement
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, fh: Option<FileHandle>, reply: ReplyAttr) {
        //TODO: Implement
    }

    fn mknod(&self, _req: &Request, parent: INodeNo, name: &OsStr, mode: u32, umask: u32, rdev: u32, reply: ReplyEntry) {
        //TODO: Implement
    }

    fn mkdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, mode: u32, umask: u32, reply: ReplyEntry) {
        //TODO: Implement
    }

    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        //TODO: Implement
    }
    
    fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        //TODO: Implement
    }

    fn rename(&self, _req: &Request, parent: INodeNo, name: &OsStr, newparent: INodeNo, newname: &OsStr, flags: RenameFlags, reply: ReplyEmpty) {
        //TODO: Implement
    }
    
    fn open(&self, _req: &Request, _ino: INodeNo, _flags: OpenFlags, reply: ReplyOpen) {
        //TODO: Implement
    }
    
    fn read(&self, _req: &Request, ino: INodeNo, fh: FileHandle, offset: u64, size: u32, 
            flags: OpenFlags, lock_owner: Option<LockOwner>, reply: ReplyData) {
        //TODO: Implement
    }

    fn write(&self, _req: &Request, ino: INodeNo, fh:FileHandle, offset: u64, data: &[u8], write_flags: WriteFlags, flags: OpenFlags, lock_owner: Option<LockOwner>, reply: ReplyWrite) {
        //TODO: Implement
    }
}

fn main () {
    println!("test");
}
