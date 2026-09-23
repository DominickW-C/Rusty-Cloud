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
    ReplyCreate,
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

use std::collections::HashMap;
use core::time;
use std::time::UNIX_EPOCH;
use std::ffi::OsStr;
use std::path::Path;
use std::fmt;

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

enum NodeKind {
    //file data
    File { data: Vec<u8> },
    //if it was directory, name and inodeno
    Directory { children: HashMap<String, u64> }
}

struct FileNode {
    attr: FileAttr,
    file_type: NodeKind
}

struct RustyCloud {
    f_node: HashMap<fuser::INodeNo, FileNode>
}

impl RustyCloud {
    pub fn new() -> Self {
        let mut f_node = HashMap::new();
        let root_attr = FileAttr {
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
        f_node.insert(fuser::INodeNo(1), FileNode {
            attr: root_attr,
            file_type: NodeKind::Directory { children: HashMap::new() }
        });

        Self { f_node }
    }
}

impl fmt::Display for RustyCloud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let root_node = INodeNo(1); 
        let root_file_node = match self.f_node.get(&root_node) {
            Some(r) => r,
            None => {
                return write!(f, "error displaying information") 
            }
        };
        let inode = root_file_node.attr.ino;
        let size = root_file_node.attr.size;
        

        write!(f, "inode: {} \n size: {}\n", inode, size)
    }
}

impl Filesystem for RustyCloud {
    //NOTE: currently this gives an error with ls, i think its becasue the file is hard coded and
    //not in any struct
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        //find the parent file
        match self.f_node.get(&parent) {
            Some(parent) => {
                //make sure the parent was a dir
                match &parent.file_type {
                    NodeKind::Directory { children } => {
                        //convert the OSstr to str
                        let name_str = match name.to_str() {
                            Some(string) => string,
                            None => {
                                println!("lookup error, coverting name to str");
                                ""
                            }
                        };
                        //find if child exists
                        match children.get(name_str) {
                            Some(inum) => {
                                let inode = INodeNo(*inum);
                                let file_attr = match self.f_node.get(&inode) {
                                    Some(r) => r.attr,
                                    None => {
                                        println!("lookup error, finding child attr");
                                        return
                                    }
                                };
                                reply.entry(&TIMEOUT, &file_attr, Generation(0))
                            },
                            None => {
                                println!("no file matching");
                                reply.error(Errno::ENOENT)
                            }
                        }
                    },
                    NodeKind::File { .. } => {
                        println!("lookup found parent as file, should never happen");
                        reply.error(Errno::ENOENT)
                    } 
                }
            },
            None => {
                println!("lookup error, could not find parent inode");
                reply.error(Errno::ENOENT)
            }
        }


        /*
        if OsStr::eq_ignore_ascii_case(name, "test") {
            println!("lookup found test file"); 
            reply.entry(&TIMEOUT, &TEST_FILE_ATTR, Generation(0)); 
        }
        println!("looking for {:?}", name);
        */
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, fh: Option<FileHandle>, reply: ReplyAttr) {
        match self.f_node.get(&ino) {
            Some(r) => {
                println!("getattr found the inode {}", ino);
                reply.attr(&TIMEOUT, &r.attr)
            }
            None => {
                reply.error(Errno::ENOENT);
            }
        };
        /*
        println!("{}", self);
        println!("inodeno: {}", ino);
        if ino == fuser::INodeNo(1) {
            reply.attr(&TIMEOUT, &ROOT_FILE_ATTR);
        } else if ino == fuser::INodeNo(2) {
            println!("getattr found test file");
            reply.attr(&TIMEOUT, &TEST_FILE_ATTR);
        } 
        else {
            reply.error(Errno::ENOENT); 
        }
        */
    }

    fn create(&self, _req: &Request, parent: INodeNo, name: &OsStr, mode: u32, umask: u32, flags: i32, reply: ReplyCreate) {
        println!("create was called");
        println!("parent INode {}", parent);
    }
    /*
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
    let mounted_fs = fuser::mount(RustyCloud::new(), mountpoint, &options);

    //only called if there was an error when mounting
    if let Err(e) = mounted_fs {
        println!("error encounted when trying to mount the filesystem");
        println!("{}", e);
    }

    
    println!("test");
}
