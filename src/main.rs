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
    FopenFlags,
    BsdFileFlags,
    TimeOrNow
};

use std::collections::HashMap;
use core::time;
use std::time::UNIX_EPOCH;
use std::ffi::OsStr;
use std::path::Path;
use std::fmt;
use std::time::SystemTime;

use std::sync::Mutex;
use std::sync::atomic::{
    AtomicU64,
    Ordering
};

mod datatypes;
use crate::datatypes::{
    FileNode,
    NodeKind,
    RustyCloud
};

const TIMEOUT: time::Duration = time::Duration::new(2, 0);

impl RustyCloud {
    pub fn new() -> Self {
        let next_inode =  Mutex::new(2);
        let free_inode = Mutex::new(Vec::new());
        let f_node = Mutex::new(HashMap::new());
        //start at 3 to avoid any in,out,err confusion
        let next_fh = AtomicU64::new(3);
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
            file_type: NodeKind::Directory { children: HashMap::new() },
            path: String::from("/") 
        });

        Self { f_node, next_fh, next_inode, free_inode }
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

fn new_inode_no(fs: &RustyCloud) -> u64 {
    let ino_num: u64;
    if fs.free_inode.lock().unwrap().is_empty() {
        //have to use a new inode number
        println!("using new inode number");
        ino_num = *fs.next_inode.lock().unwrap();
        *fs.next_inode.lock().unwrap() += 1;
    } else {
        //have an available inode, use that 
        println!("using available Inode number");
        let new_inode_index = fs.free_inode.lock().unwrap().len() - 1;
        ino_num = fs.free_inode.lock().unwrap()[new_inode_index];
        fs.free_inode.lock().unwrap().pop();
    }
    ino_num
}

fn new_file_node(new_attr: &FileAttr, parent_path: &String, node_name: &String) -> Option<FileNode> {
    let mut file_path = parent_path.clone() + &node_name;    
    let new_f_node: FileNode;

    match new_attr.kind {
        FileType::Directory => {
            file_path = file_path.clone() + &"/";
            new_f_node = FileNode {
                attr: *new_attr,
                file_type: NodeKind::Directory { children: HashMap::new() },
                path: file_path 
            };
        } 
        FileType::RegularFile => {
            new_f_node = FileNode {
                attr: *new_attr,
                file_type: NodeKind::File { file_data: Vec::new() },
                path: file_path 
            };
        } 
        FileType::NamedPipe | FileType::CharDevice | FileType::BlockDevice | FileType::Symlink | FileType::Socket => {
            println!("ERROR, BAD THING HAPPENED");
            println!("NOT IMPLEMENTED FOR NAMED PIPES, CHAR DEVICES, etc");
            println!("SEE new_file_node");
            return None
        }

    }
    Some(new_f_node)
}

impl Filesystem for RustyCloud {
    //not in any struct
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        println!("calling lookup for parent: {:?} name {:?}", parent, name);
        //find the parent file
        let mut node_map = self.f_node.lock().unwrap();
        match node_map.get(&parent) {
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
                        match children.get(name_str) {
                            Some(inum) => {
                                let inode = INodeNo(*inum);
                                let file_node = match node_map.get_mut(&inode) {
                                    Some(r) => r,
                                    None => {
                                        println!("lookup error, finding child attr");
                                        return reply.error(Errno::ENOENT);
                                    }
                                };
                                println!("DEBUG: lookup filetype: {:?}", file_node.attr.kind);

                                //if it was a file, we want to update the size
                                //NOTE: can probably clean up
                                match &file_node.file_type {
                                    NodeKind::File { file_data } => {
                                        file_node.attr.size = file_data.len() as u64;
                                    }
                                    NodeKind::Directory { .. } => {}
                                };

                                reply.entry(&TIMEOUT, &file_node.attr, Generation(0))
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
        println!("calling getattr for inode {:?}", ino);
        let mut node_map = self.f_node.lock().unwrap();
        let node = node_map.get_mut(&ino).unwrap();
        match &mut node.file_type {
            NodeKind::File { file_data } => {
                println!("getattr thinks is file");
                node.attr.size = file_data.len() as u64;
                println!("the length of the file is now {}", node.attr.size);
                println!("DEBUG: getattr filetype: {:?}", node.attr.kind);
                reply.attr(&TIMEOUT, &node.attr)
            }
            NodeKind::Directory { .. } => {
                println!("getattr thinks is dir");
                println!("DEBUG: getattr filetype: {:?}", node.attr.kind);
                reply.attr(&TIMEOUT, &node.attr)
            }
        }
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

    fn setattr(&self, _req: &Request, ino: INodeNo, mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, size: Option<u64>,
               _attime: Option<TimeOrNow>, _mtime: Option<TimeOrNow>, _ctime: Option<SystemTime>, fh: Option<FileHandle>, _crtime: Option<SystemTime>,
               _chgtime: Option<SystemTime>, _bkuptime: Option<SystemTime>, flags: Option<BsdFileFlags>, reply: ReplyAttr) {

        println!("calling setattr");
        println!("size was {:?}", size);
        let mut hash_m = self.f_node.lock().unwrap();
        //NOTE:: could be a pointless match
        //TODO: add to this
        match hash_m.get_mut(&ino) {
            Some(node) => {
                //size can be none
                match size {
                    Some(re) => {
                        node.attr.size = re;
                        match &mut node.file_type {
                            NodeKind::File { file_data } => {
                                file_data.resize(re as usize, 0);                  
                                reply.attr(&TIMEOUT, &node.attr)
                            }
                            NodeKind::Directory { .. } => reply.attr(&TIMEOUT, &node.attr)
                        }
                    }
                    None => reply.attr(&TIMEOUT, &node.attr)
                }
            }
            None => {
                println!("setattr error, to inode found matching");
                reply.error(Errno::ENOENT);
            }
        }
        //TODO: truncate will be called to this, and it needs to set a file size to 0
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
        match hash_m.get_mut(&parent) {
            Some (p_result) => {
                match &mut p_result.file_type {
                    NodeKind::Directory { children } => {
                        //TODO: clean up var names
                        let str_name =  name.to_str().unwrap();
                        let str_ret = String::from(str_name);
                        let ino_num: u64 = new_inode_no(self);
                        children.insert(str_ret.clone(), ino_num);
                                        
                        let new_file_attr: fuser::FileAttr = FileAttr {
                            ino: INodeNo(ino_num),
                            size: 0,
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
                        let new_f_node: FileNode = new_file_node(&new_file_attr, &p_result.path.clone(), &str_ret).unwrap();
                        hash_m.insert(INodeNo(ino_num), new_f_node);
                        

                        //WARNING: might not like 0 as file handle
                        reply.created(&TIMEOUT, &new_file_attr, Generation(0), FileHandle(0), FopenFlags::empty())
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

    fn mkdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, mode: u32, umask: u32, reply: ReplyEntry) {
        println!("calling mkdir");

        let mut node_map = self.f_node.lock().unwrap();
        match node_map.get_mut(&parent) {
            Some(parent) => {
                match &mut parent.file_type {
                    NodeKind::Directory { children } => {
                        let str_name =  name.to_str().unwrap();
                        let str_ret = String::from(str_name);
                        let ino_num: u64 = new_inode_no(self); 
                        children.insert(str_ret.clone(), ino_num);

                        let new_directory_attr: fuser::FileAttr = FileAttr {
                            ino: INodeNo(ino_num),
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
                        let new_d_node: FileNode = new_file_node(&new_directory_attr, &parent.path.clone(), &str_ret).unwrap();

                        node_map.insert(INodeNo(ino_num), new_d_node);
                        reply.entry(&TIMEOUT, &new_directory_attr, Generation(0));
                    }
                    NodeKind::File { .. } => {
                        println!("parent was a file, not possible");
                        reply.error(Errno::ENOENT);
                    }
                }

            }
            None => {
                println!("could not find the parent node, should never happen");
                reply.error(Errno::ENOENT);
            }
        }
    }
    /*
       fn mknod(&self, _req: &Request, parent: INodeNo, name: &OsStr, mode: u32, umask: u32, rdev: u32, reply: ReplyEntry) {
    //TODO: Implement
    println!("calling mknod");
       }

       fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
    //TODO: Implement
    println!("calling unlink");
       }

       fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
    //TODO: Implement
    println!("calling rmdir");
       }

*/
    fn rename(&self, _req: &Request, parent: INodeNo, name: &OsStr, newparent: INodeNo, 
              newname: &OsStr, flags: RenameFlags, reply: ReplyEmpty) {
        println!("calling rename");

        let mut node_map = self.f_node.lock().unwrap(); 
        //parent will always be a dir and SHOULD always exist
        let dir_node = node_map.get_mut(&parent).unwrap();
        //parent stays same, rename the file
        if parent == newparent {
            println!("file moved in same parent dir");
            match &mut dir_node.file_type {
                NodeKind::Directory { children } => {
                    let old_name: String = String::from(name.to_str().unwrap());
                    let new_name: String = String::from(newname.to_str().unwrap());
                    match children.get_mut(&old_name) {
                        Some(_) => {
                            let ino: u64 = children.remove(&old_name).unwrap(); 
                            children.insert(new_name, ino);
                            reply.ok()
                        } 
                        None => {
                            println!("could not find file to rename");
                            reply.error(Errno::ENOENT)
                        }
                    }
                }
                NodeKind::File { .. } => {
                    println!("ERROR rename: should never happen, file in file");
                    reply.error(Errno::ENOENT)

                }
            }

                
        } else {
            println!("file moved to new dir");
        }
    }

    fn open(&self, _req: &Request, _ino: INodeNo, flags: OpenFlags, reply: ReplyOpen) {
        println!("called open");
        println!("flags are {:?}", flags);

        let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
        reply.opened(FileHandle(fh), FopenFlags::empty());
    }

    fn read(&self, _req: &Request, ino: INodeNo, fh: FileHandle, offset: u64, size: u32, 
        flags: OpenFlags, lock_owner: Option<LockOwner>, reply: ReplyData) {
        //TODO: Implement
        //NOTE: trying to read a file with size 0 does absolutely nothing
        println!("read called from from ino: {:?}", ino);

        let node_map = self.f_node.lock().unwrap();
        //file def exists, just unwrap
        let file_node = node_map.get(&ino).unwrap();
        match &file_node.file_type {
            NodeKind::File { file_data }=> {
                println!("file is of size {}", file_node.attr.size);
                println!("replying with {:?} as data", file_data);
                reply.data(file_data);   
            }
            NodeKind::Directory { .. }=> {
                println!("read error, reading a dir");
                reply.error(Errno::ENOENT);
            }
        }; 

        /*
           println!("reading ");
           if ino == INodeNo(2) {
           println!("opening test file");
           let test_output = b"something";
           reply.data(test_output);
           }
           */
    }

    fn readdir(&self, _req: &Request, ino: INodeNo, fh: FileHandle, offset: u64, mut reply: ReplyDirectory) {
        //TODO: Break out of loop if add is full? and then call this function again
        println!("calling readdir on inode {}", ino);
        println!("what is the offset? {}", offset);
        //ino is the current directory we are in
        //first get the parent 

        //hashmap of f_nodes
        let node_map = &self.f_node.lock().unwrap();
        //parent node
        let parent_node = node_map.get(&ino).unwrap();

        match &parent_node.file_type {
            NodeKind::Directory { children } => {
                let child_files = children.clone();
                //string (name), u64 (inode)
                for (i, (name, ino)) in child_files.iter().enumerate().skip(offset as usize) {
                    println!("in loop, name: {}", name);
                    println!("in loop, ino: {}", ino);
                    //now get the type of file it is 
                    let file_node = node_map.get(&INodeNo(*ino)).unwrap();
                    let ft = match file_node.file_type {
                        NodeKind::Directory { .. } => FileType::Directory, 
                        NodeKind::File { .. } => FileType::RegularFile 
                    };
                    let buffer_full = reply.add(INodeNo(*ino), (i + 1) as u64, ft, name);
                    if buffer_full {
                        break;
                    }


                }
                reply.ok();
            }
            NodeKind::File { .. } => {
                println!("not possible");
                reply.error(Errno::ENOENT);
            }
        }


        /*
           if offset == 0 {
           let _ = reply.add(INodeNo(2), 1, FileType::RegularFile, "test");
           }
           */
        //doing reply.ok with no add causes it to stop
    }

    fn write(&self, _req: &Request, ino: INodeNo, fh:FileHandle, offset: u64, data: &[u8], 
        write_flags: WriteFlags, flags: OpenFlags, lock_owner: Option<LockOwner>, reply: ReplyWrite) {
        //TODO: Implement
        println!("calling write");
        println!("write flags are: {:?}", write_flags);

        //make sure that we are writing to a file not a dir
        let mut node_map = self.f_node.lock().unwrap();
        match node_map.get_mut(&ino) {
            Some(node) => {
                match &mut node.file_type {
                    NodeKind::File { file_data } => {
                        for ch in data.iter() {
                            file_data.push(*ch);
                        }
                        reply.written(data.len() as u32);
                    }
                    NodeKind::Directory { .. } => {
                        println!("can't write to a dir");
                        reply.error(Errno::ENOENT);
                    }
                } 
            }
            None => {
                println!("write error, could not find file in system");
                reply.error(Errno::ENOENT);
            }
        };
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
