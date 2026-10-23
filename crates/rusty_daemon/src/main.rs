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
    TimeOrNow,
};

use sha2::{Sha256, Digest};

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

mod merging;
use crate::merging:: {
    split_by_changes,
    merge_needed,
    merge_file
};

mod helpers;
use crate::helpers::{
    unwrap_parent_dir,
    new_inode_no,
    new_file_node,
};

mod datatypes;
use crate::datatypes::{
    FileNode,
    NodeKind,
    FileGrouped,
    GroupType,
    RustyCloud, 
};

const TIMEOUT: time::Duration = time::Duration::new(2, 0);

impl RustyCloud {
    pub fn new() -> Self {
        let next_inode =  Mutex::new(2);
        let free_inode = Mutex::new(Vec::new());
        let f_node = Mutex::new(HashMap::new());
        let splits = Mutex::new(HashMap::new());
        //start at 3 to avoid any in,out,err confusion
        let next_fh = AtomicU64::new(3);
        let root_attr = FileAttr {
            ino: INodeNo(1),
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH, kind: FileType::Directory, perm: 755,
            nlink: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0
        };
        f_node.lock().unwrap().insert(fuser::INodeNo(1), FileNode {
            attr: root_attr,
            file_type: NodeKind::Directory { children: HashMap::new() },
            path: String::from("/"),
            hash: None,
        });

        Self { f_node, next_fh, next_inode, free_inode, splits }
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

impl Filesystem for RustyCloud {

    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        println!("calling lookup for parent: {:?} name {:?}", parent, name);
        //find the parent file
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let name_str = name.to_str().unwrap();

        //find if child exists
        match children.get(name_str) {
            Some(inum) => {
                let inode = INodeNo(*inum);
                let file_node = node_map.get_mut(&inode).unwrap();
                //if it was a file, we want to update the size
                //NOTE: can probably clean up
                match &file_node.file_type {
                    NodeKind::File { file_data } => {
                        file_node.attr.size = file_data.len() as u64;
                    }
                    NodeKind::Directory { .. } => {},
                    //TODO: filesize might be wrong for symlinksj
                    NodeKind::Symlink { .. } => {
                    }

                };
                reply.entry(&TIMEOUT, &file_node.attr, Generation(0))
            },
            None => {
                println!("no file matching");
                reply.error(Errno::ENOENT)
            }
        }

    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        println!("calling getattr for inode {:?}", ino);
        let mut node_map = self.f_node.lock().unwrap();
        let node = node_map.get_mut(&ino).unwrap();
        match &mut node.file_type {
            NodeKind::File { file_data } => {
                node.attr.size = file_data.len() as u64;
                reply.attr(&TIMEOUT, &node.attr)
            }
            NodeKind::Directory { .. } => {
                reply.attr(&TIMEOUT, &node.attr)
            }
            NodeKind::Symlink { .. } => {
                reply.attr(&TIMEOUT, &node.attr)
            }
        }
    }

    fn setattr(&self, _req: &Request, ino: INodeNo, _mode: Option<u32>, _uid: Option<u32>, _gid: Option<u32>, size: Option<u64>,
        _attime: Option<TimeOrNow>, _mtime: Option<TimeOrNow>, _ctime: Option<SystemTime>, _fh: Option<FileHandle>, 
        _crtime: Option<SystemTime>, _chgtime: Option<SystemTime>, _bkuptime: Option<SystemTime>, _flags: Option<BsdFileFlags>, reply: ReplyAttr) {

        println!("calling setattr");
        let mut hash_m = self.f_node.lock().unwrap();
        let node = hash_m.get_mut(&ino).unwrap();
        //size can be none
        //NOTE: Check to see what is happening here
        match size {
            Some(re) => {
                node.attr.size = re;
                match &mut node.file_type {
                    NodeKind::File { file_data } => {
                        file_data.resize(re as usize, 0);                  
                        reply.attr(&TIMEOUT, &node.attr)
                    }
                    NodeKind::Directory { .. } => reply.attr(&TIMEOUT, &node.attr),
                    NodeKind::Symlink { .. } => reply.attr(&TIMEOUT, &node.attr)
                }
            }
            None => reply.attr(&TIMEOUT, &node.attr)
        }
    }

    fn create(&self, _req: &Request, parent: INodeNo, name: &OsStr, _mode: u32, _umask: u32, _flags: i32, reply: ReplyCreate) {
        println!("calling create");
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
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
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0
        };
        //WARNING: might be inefficient but code is a lot better
        let p_result = node_map.get(&parent).unwrap();
        let new_f_node: FileNode = new_file_node(&new_file_attr, &p_result.path.clone(), &str_ret).unwrap();
        node_map.insert(INodeNo(ino_num), new_f_node);


        let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
        reply.created(&TIMEOUT, &new_file_attr, Generation(0), FileHandle(fh), FopenFlags::empty())

    }

    fn mkdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, _mode: u32, _umask: u32, reply: ReplyEntry) {
        println!("calling mkdir");

        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        //TODO: better var names
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
            nlink: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0
        };        
        //WARNING: might be inefficient but code is a lot better
        let p_result = node_map.get(&parent).unwrap();
        let new_d_node: FileNode = new_file_node(&new_directory_attr, &p_result.path.clone(), &str_ret).unwrap();

        node_map.insert(INodeNo(ino_num), new_d_node);
        reply.entry(&TIMEOUT, &new_directory_attr, Generation(0));

    }

    fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        println!("calling rmdir");

        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);

        let name_string = String::from(name.to_str().unwrap());
        let ino_num = children.remove(&name_string).unwrap();
        node_map.remove(&INodeNo(ino_num));
        self.free_inode.lock().unwrap().push(ino_num);
        reply.ok();
    }

    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        println!("calling unlink");

        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let name_string = String::from(name.to_str().unwrap());
        let ino_num = children.remove(&name_string).unwrap();

        //removed from the parent, although if it was a link we want to 
        //keep the other reference to it
        let file_node = node_map.get_mut(&INodeNo(ino_num)).unwrap();
        if file_node.attr.nlink > 1 {
            file_node.attr.nlink -= 1;
        } else {
            node_map.remove(&INodeNo(ino_num));
            self.free_inode.lock().unwrap().push(ino_num);
        }
        reply.ok();
    }

    fn rename(&self, _req: &Request, parent: INodeNo, name: &OsStr, newparent: INodeNo, 
        newname: &OsStr, _flags: RenameFlags, reply: ReplyEmpty) {
        println!("calling rename");

        let mut node_map = self.f_node.lock().unwrap(); 
        let children = unwrap_parent_dir(&mut node_map, parent);
        let old_name: String = String::from(name.to_str().unwrap());
        let new_name: String = String::from(newname.to_str().unwrap());
        match children.get_mut(&old_name) {
            Some(_) => {
                let ino: u64 = children.remove(&old_name).unwrap(); 
                if parent == newparent {
                    children.insert(new_name, ino);
                    reply.ok()
                } else {
                    let new_children = unwrap_parent_dir(&mut node_map, newparent);
                    new_children.insert(new_name, ino);
                    reply.ok();
                }
            } 
            None => {
                println!("could not find file to rename");
                reply.error(Errno::ENOENT)
            }
        }


    }

    fn open(&self, _req: &Request, _ino: INodeNo, _flags: OpenFlags, reply: ReplyOpen) {
        println!("called open");
        let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
        reply.opened(FileHandle(fh), FopenFlags::empty());
    }

    fn read(&self, _req: &Request, ino: INodeNo, _fh: FileHandle, _offset: u64, _size: u32, 
        _flags: OpenFlags, _lock_owner: Option<LockOwner>, reply: ReplyData) {
        println!("read called from from ino: {:?}", ino);

        let node_map = self.f_node.lock().unwrap();
        //file def exists, just unwrap
        let file_node = node_map.get(&ino).unwrap();
        match &file_node.file_type {
            NodeKind::File { file_data } => {
                reply.data(file_data);   
            }
            NodeKind::Directory { .. } => {
                println!("read error, reading a dir");
                reply.error(Errno::ENOENT);
            }
            NodeKind::Symlink { .. } => {
                //TODO: read a symlink
                //will need to access the symlink data so dont leave ..
                println!("TODO: read the symlink parent data");

            }
        }; 
    }

    fn readdir(&self, _req: &Request, ino: INodeNo, _fh: FileHandle, offset: u64, mut reply: ReplyDirectory) {
        //TODO: Break out of loop if add is full? and then call this function again
        println!("calling readdir on inode {}", ino);
        //ino is the current directory we are in
        //first get the parent 

        //hashmap of f_nodes
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, ino);
        let child_files = children.clone();
        //string (name), u64 (inode)
        for (i, (name, ino)) in child_files.iter().enumerate().skip(offset as usize) {
            //now get the type of file it is 
            let file_node = node_map.get(&INodeNo(*ino)).unwrap();
            let ft = match file_node.file_type {
                NodeKind::Directory { .. } => FileType::Directory, 
                NodeKind::File { .. } => FileType::RegularFile,
                NodeKind::Symlink { .. } => FileType::Symlink 
            };
            let buffer_full = reply.add(INodeNo(*ino), (i + 1) as u64, ft, name);
            if buffer_full {
                break;
            }


        }
        reply.ok();
    }

    fn write(&self, _req: &Request, ino: INodeNo, _fh:FileHandle, _offset: u64, data: &[u8], 
        _write_flags: WriteFlags, _flags: OpenFlags, _lock_owner: Option<LockOwner>, reply: ReplyWrite) {

        println!("calling write");

        //make sure that we are writing to a file not a dir
        let mut node_map = self.f_node.lock().unwrap();
        let mut splits = self.splits.lock().unwrap();
        match node_map.get_mut(&ino) {
            Some(node) => {
                match &mut node.file_type {
                    NodeKind::File { file_data } => {
                        //TODO: when something truncates a file the data by here is empty 
                        let old_data = file_data.clone(); 
                        let old_hash = node.hash.unwrap();
                        //generates the new hash
                        let file_hash = Sha256::digest(data); 

                        match splits.get(&old_hash) {
                            Some(old_file_group) => {
                                let new_file_group: FileGrouped = split_by_changes(old_data, data.to_vec(), file_hash);
                                let new_file_data: Vec<u8> = merge_file(old_file_group, &new_file_group);
                                println!("new file data after merge was {:?}", new_file_data);

                                splits.remove(&old_hash);
                                splits.insert(file_hash, new_file_group);

                                for ch in new_file_data.iter() {
                                    file_data.push(*ch);
                                }
                                reply.written(new_file_data.len() as u32);
                            },
                            None => {
                                //TODO: we are looking for a different split causing an error, find
                                //the correct split to look for
                                println!("assuming first file write");
                                println!("our splits is {:?}", splits.keys());
                                println!("we were looking for {:?}", old_hash);
                                let base_data = vec![data.to_vec()];
                                let group_types = vec![GroupType::Same];
                                let base_group = FileGrouped {
                                    data_group: base_data,
                                    group_type: group_types,
                                    group_breaks: Vec::new(),
                                    made_from: file_hash
                                };
                                splits.insert(file_hash, base_group);
                                for ch in data.iter() {
                                    file_data.push(*ch);
                                }

                                reply.written(data.len() as u32);

                            }
                        }

                        //if I write to a file, take the old hash and look for it in the splits
                        //hashmap, if it is not found, for now assume that it is a new file and its
                        //the first write, if it was found, split the new data, and compare that
                        //split to the split saved for the old file. after delete the split for the
                        //old file, and insert the split for the new one

                        /*
                           if merge_needed(node.hash.unwrap(), file_hash) {
                           node.hash = Some(file_hash);
                           println!("new file hash is now {:?}", file_hash);

                           }
                           */

                    }
                    NodeKind::Directory { .. } => {
                        println!("WRITE: can't write to a dir");
                        reply.error(Errno::ENOENT);
                    }
                    NodeKind::Symlink { .. } => {
                        println!("WRITE: can't write to symlink");
                        reply.error(Errno::ENOENT)
                    }
                } 
            }
            None => {
                println!("write error, could not find file in system");
                reply.error(Errno::ENOENT);
            }
        };
    }

    fn link(&self, _req: &Request, ino: INodeNo, newparent: INodeNo, newname: &OsStr, reply: ReplyEntry)  {
        println!("making a hard link");
        let mut node_map = self.f_node.lock().unwrap();
        // ino is the file we are trying to link to a new parent
        // needs to be mutable to increment the links
        let file_node = node_map.get_mut(&ino).unwrap();
        file_node.attr.nlink += 1;
        let attr_copy = file_node.attr;
        if matches!(file_node.file_type, NodeKind::File { .. }) {
            //good to create the link
            let name_str = newname.to_str().unwrap();
            let ret_str = String::from(name_str);
            let children = unwrap_parent_dir(&mut node_map, newparent);
            //now the file exists in the new parent
            children.insert(ret_str, ino.0);
            reply.entry(&TIMEOUT, &attr_copy, Generation(0));

        } else {
            println!("hard link error, not a file");
            reply.error(Errno::ENOENT);
        }
    }

    fn symlink (&self, _req: &Request, parent: INodeNo, link_name: &OsStr, target: &Path, reply: ReplyEntry) {
        let mut node_map = self.f_node.lock().unwrap();
        println!("parent is {:?}", parent);

        let sym_ino: u64 = new_inode_no(self);
        let children = unwrap_parent_dir(&mut node_map, parent);
        let sym_str: &str = link_name.to_str().unwrap();
        children.insert(String::from(sym_str), sym_ino);

        //now the sym link exists in parent, add to node_map 
        let sym_attr = FileAttr {
            ino: INodeNo(sym_ino),
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime:UNIX_EPOCH,
            kind: FileType::Symlink,
            perm: 755,
            nlink: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0
        };
        let p_node = node_map.get(&parent).unwrap();
        let sym_node = FileNode {
            attr: sym_attr,
            file_type: NodeKind::Symlink { target_ino: String::from(target.to_str().unwrap()) }, 
            path: p_node.path.clone() + sym_str,
            hash: None,
        };
        node_map.insert(INodeNo(sym_ino), sym_node);
        reply.entry(&TIMEOUT, &sym_attr, Generation(0));
    }

    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
        println!("calling readlink");
        let node_map = self.f_node.lock().unwrap();
        let link_node = &node_map.get(&ino).unwrap().file_type; 

        if let NodeKind::Symlink { target_ino } = link_node {
            println!("readlink is replying with {:?}", target_ino);
            reply.data(target_ino.as_bytes());
        } else {
            println!("tried reading non symlink as symlink");
            reply.error(Errno::ENOENT)
        }
    }

    fn release(&self, _req: &Request, _ino: INodeNo, _fh: FileHandle, _flags: OpenFlags, _lock_owner: Option<LockOwner>,
        _flush: bool, reply: ReplyEmpty) {
        println!("releasing a file");
        reply.ok()
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
