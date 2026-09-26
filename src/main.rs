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
    Generation,
    FopenFlags
};

use std::collections::HashMap;
use core::time;
use std::time::UNIX_EPOCH;
use std::ffi::OsStr;
use std::path::Path;
use std::fmt;

use std::sync::Mutex;


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
    Directory { children: Mutex<HashMap<String, u64>> }
}

struct FileNode {
    attr: FileAttr,
    file_type: NodeKind,
    path: String
}

struct RustyCloud {
    f_node: Mutex<HashMap<fuser::INodeNo, FileNode>>,
    next_inode: Mutex<u64>,
    free_inode: Mutex<Vec<u64>>
}

impl RustyCloud {
    pub fn new() -> Self {
        let next_inode =  Mutex::new(2);
        let free_inode = Mutex::new(Vec::new());
        let f_node = Mutex::new(HashMap::new());
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
        f_node.lock().unwrap().insert(fuser::INodeNo(1), FileNode {
            attr: root_attr,
            file_type: NodeKind::Directory { children: Mutex::new(HashMap::new()) },
            path: String::from("/") 
        });

        Self { f_node, next_inode, free_inode }
    }
}

impl fmt::Display for RustyCloud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let root_node = INodeNo(1); 
        let hash_m = self.f_node.lock().unwrap();
        let root_file_node = match hash_m.get(&root_node) {
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


/*
 * NOTE: work in progress, trying to figure out how to make this useful
fn find_parent_node(fs: RustyCloud, parent_ino: INodeNo) -> Option<FileNode> {
    match fs.f_node.get(&parent_ino) {
        Some(parent) => {
            match &parent.file_type {
                NodeKind::Directory { .. } => {
                    Some(*parent)
                },
                NodeKind::File { .. } => {
                    println!("parent was a file, this should never happen");
                    None
                }
            }
        },
        None => {
            println!("error in find_parent_node");
            println!("could not find parent f_node");
            println!("parent ino {}", parent_ino);
            None
        }
    }

}
*/

impl Filesystem for RustyCloud {
    //TODO: implement
    fn setattr() {

    }

    //NOTE: currently this gives an error with ls, i think its becasue the file is hard coded and
    //not in any struct
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        //find the parent file
        let hash_m = self.f_node.lock().unwrap();
        match hash_m.get(&parent) {
            Some(parent) => {
                //make sure the parent was a dir a
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
                        match children.lock().unwrap().get(name_str) {
                            Some(inum) => {
                                let inode = INodeNo(*inum);
                                let hash_m = self.f_node.lock().unwrap();
                                let file_attr = match hash_m.get(&inode) {
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
        let hash_m = self.f_node.lock().unwrap();
        match hash_m.get(&ino) {
            Some(r) => {
                println!("getattr found the inode {}", ino);
                reply.attr(&TIMEOUT, &r.attr)
            }
            None => {
                reply.error(Errno::ENOENT);
            }
        };
        /*
        println!("{}  let root_file_node = match self.f_node.get(&root_node) {

            Some(r) => r,

            None => write!(f, "error")

        }; ", self);
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
        println!("parent INode {}", parent);
        println!("name of file is {:?}", name);
        println!("mode is {}", mode);
        println!("umask is {}", umask);
        println!("flags is {}", flags);
        //go to the parent node, create a child for it
        //WARNING: DID THIS CREATE A LOCAL
        let mut hash_m = self.f_node.lock().unwrap();
        match hash_m.get(&parent) {
            Some (p_result) => {
                match &p_result.file_type {
                    NodeKind::Directory { children } => {
                        let str_name =  name.to_str().unwrap();
                        let str_ret = String::from(str_name);
                        let ino_num: u64;
                        if self.free_inode.lock().unwrap().is_empty() {
                            //have to use a new inode number
                            println!("using new inode number");
                            ino_num = *self.next_inode.lock().unwrap();
                            children.lock().unwrap().insert(str_ret.clone(), ino_num);
                            *self.next_inode.lock().unwrap() += 1;
                        } else {
                            //have an available inode, use that 
                            println!("using available Inode number");
                            let new_inode_index = self.free_inode.lock().unwrap().len() - 1;
                            ino_num = self.free_inode.lock().unwrap()[new_inode_index];
                            children.lock().unwrap().insert(str_ret.clone(), ino_num);
                            self.free_inode.lock().unwrap().pop();
                        }

                        //should not have to worry about / since a directory will be made with one
                        //at end 
                        let parent_path = &p_result.path.clone();
                        let file_path = parent_path.clone() + &str_ret;

                        let new_file_node = FileNode {
                                attr: TEST_FILE_ATTR,
                                file_type: NodeKind::File { data: Vec::new() },
                                path: file_path 
                        };
                        hash_m.insert(INodeNo(ino_num), new_file_node);
                        //WARNING: might not like 0 as file handle
                        reply.created(&TIMEOUT, &TEST_FILE_ATTR, Generation(0), FileHandle(0), FopenFlags::empty())
                    } 
                    NodeKind::File { .. } => {
                        println!("parent was a file, should never happen");
                        reply.error(Errno::ENOENT);
                    }
                }    
            }
            None => {
                println!("parent node not found");
                reply.error(Errno::ENOENT);
            }
        }

        //create a new node for the file, updates its path




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
