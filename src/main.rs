use fuser::{
    Filesystem,
    Request,
    INodeNo,
    FileHandle,
    FileAttr,
    FileType,
    RenameFlags,
    OpenFlags,
    LockOwner,
    ReplyData,
    ReplyAttr,
    ReplyEmpty,
    ReplyOpen,
    ReplyWrite,
    ReplyDirectory,
    ReplyEntry,
    WriteFlags,
    MountOption,
    Config,
    SessionACL,
    Errno,
    Generation
};


use core::time;
use std::time::UNIX_EPOCH;
use std::ffi::OsStr;
use std::path::Path;
use libc;

//TESTING PURPOSES
//see documentation for figuring out what these are again
const ROOT_FILE_ATTR: fuser::FileAttr = FileAttr {
    ino: INodeNo(1),
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH,
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime:UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 755,
    nlink: 0,
    uid: 0,
    gid: 0,
    rdev: 0,
    blksize: 4096,
    flags: 0
};

const TEST_FILE_ATTR: fuser::FileAttr = FileAttr {
    ino: INodeNo(2),
    size: 9,
    blocks: 0,
    atime: UNIX_EPOCH,
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime:UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 755,
    nlink: 0,
    uid: 0,
    gid: 0,
    rdev: 0,
    blksize: 4096,
    flags: 0
};

const TIMEOUT: time::Duration = time::Duration::new(2, 0);

struct RustyCloud;
impl Filesystem for RustyCloud {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        if OsStr::eq_ignore_ascii_case(name, "test") {
            println!("lookup found test file"); 
            reply.entry(&TIMEOUT, &TEST_FILE_ATTR, Generation(0)); 
        }
        println!("looking for {:?}", name);
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, fh: Option<FileHandle>, reply: ReplyAttr) {
        //TODO: Fix time, currently everything is the current time
        if ino == fuser::INodeNo(1) {
            reply.attr(&TIMEOUT, &ROOT_FILE_ATTR);
        } else if ino == fuser::INodeNo(2) {
            println!("getattr found test file");
            reply.attr(&TIMEOUT, &TEST_FILE_ATTR);
        } 
        else {
            reply.error(Errno::ENOENT); 
        }
    }

    fn mknod(&self, _req: &Request, parent: INodeNo, name: &OsStr, mode: u32, umask: u32, rdev: u32, reply: ReplyEntry) {
        //TODO: Implement
        println!("calling mknod");
    }

    fn mkdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, mode: u32, umask: u32, reply: ReplyEntry) {
        //TODO: Implement
        println!("calling mkdir");
    }

    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        //TODO: Implement
        println!("calling unlink");
    }
    
    fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        //TODO: Implement
        println!("calling rmdir");
    }

    fn rename(&self, _req: &Request, parent: INodeNo, name: &OsStr, newparent: INodeNo, 
              newname: &OsStr, flags: RenameFlags, reply: ReplyEmpty) {
        //TODO: Implement
        println!("calling rename");
    }
    
    /*
    fn open(&self, _req: &Request, _ino: INodeNo, _flags: OpenFlags, reply: ReplyOpen) {
        //TODO: Implement
        println!("calling open");
    }
    */
    
    fn read(&self, _req: &Request, ino: INodeNo, fh: FileHandle, offset: u64, size: u32, 
            flags: OpenFlags, lock_owner: Option<LockOwner>, reply: ReplyData) {
        //TODO: Implement
        //NOTE: trying to read a file with size 0 does absolutely nothing
        println!("reading ");
        if ino == INodeNo(2) {
            println!("opening test file");
            let test_output = b"something";
            reply.data(test_output);
        }
    }

    fn readdir(&self, _req: &Request, ino: INodeNo, fh: FileHandle, offset: u64, mut reply: ReplyDirectory) {
        if offset == 0 {
            let _ = reply.add(INodeNo(2), 64, FileType::RegularFile, "test");
        }
        //doing reply.ok with no add causes it to stop
        reply.ok();
    }

    fn write(&self, _req: &Request, ino: INodeNo, fh:FileHandle, offset: u64, data: &[u8], 
             write_flags: WriteFlags, flags: OpenFlags, lock_owner: Option<LockOwner>, reply: ReplyWrite) {
        //TODO: Implement
        println!("calling write");
    }
}

fn main () {
    //TODO: if the directory does not exist, make it
    //path where the file will be mounted
    let mountpoint = Path::new("/home/domc/rusty_cloud/fs_test");
    //sets default options
    let mut options = Config::default();
    //changes the mount_options to auto unmount
    options.mount_options = vec![
            MountOption::AutoUnmount
    ];
    //lets any user access the filesystem
    options.acl = SessionACL::All;

    //mounts the filesystem
    let mounted_fs = fuser::mount(RustyCloud, mountpoint, &options);

    //only called if there was an error when mounting
    if let Err(e) = mounted_fs {
        println!("error encounted when trying to mount the filesystem");
        println!("{}", e);
    }

    
    println!("test");
}
