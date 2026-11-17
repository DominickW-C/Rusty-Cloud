#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use fuser::{
    Filesystem, Request, INodeNo, FileHandle, FileAttr, FileType, RenameFlags, OpenFlags,
    LockOwner, ReplyData, ReplyAttr, ReplyEmpty, ReplyOpen, ReplyCreate, ReplyWrite,
    ReplyDirectory, ReplyEntry, WriteFlags, MountOption, Config, SessionACL, Errno,
    Generation, FopenFlags, BsdFileFlags, TimeOrNow,
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
use std::sync::atomic::{AtomicU64, Ordering};
use std::env;
mod merging {
    use crate::datatypes::{FileGrouped, GroupType};
    use sha2::Sha256;
    fn find_file_lines(file: &[u8]) -> Vec<Vec<u8>> {
        let mut lines: Vec<Vec<u8>> = Vec::new();
        let mut new_line: Vec<u8> = Vec::new();
        for char in file.iter() {
            if *char != 10 {
                new_line.push(*char);
            } else {
                new_line.push(*char);
                lines.push(new_line.clone());
                new_line.clear();
            }
        }
        lines
    }
    pub fn split_by_changes(
        old_file_group: &FileGrouped,
        new_file_data: Vec<u8>,
        from_hash: sha2::digest::Output<Sha256>,
    ) -> FileGrouped {
        let old_file_data = regroup(old_file_group);
        let old_by_line: Vec<Vec<u8>> = find_file_lines(&old_file_data);
        {
            ::std::io::_print(format_args!("old by line was {0:?}\n", old_by_line));
        };
        let new_by_line: Vec<Vec<u8>> = find_file_lines(&new_file_data);
        {
            ::std::io::_print(format_args!("new by line was {0:?}\n", new_by_line));
        };
        let mut old_iter = 0;
        let mut new_iter = 0;
        let mut text_group = FileGrouped {
            data_group: Vec::new(),
            group_type: Vec::new(),
            group_breaks: Vec::new(),
            made_from: from_hash,
        };
        let mut first_line_flag: bool = true;
        let mut new_group: Vec<u8> = Vec::new();
        if old_file_group.group_type.is_empty() {
            return text_group;
        }
        {
            ::std::io::_print(format_args!("old iter is now {0}\n", old_iter));
        };
        {
            ::std::io::_print(
                format_args!(
                    "what are our old data groups? {0:?}\n",
                    old_file_group.data_group,
                ),
            );
        };
        while old_iter < old_by_line.len() || new_iter < new_by_line.len() {
            if old_iter >= old_by_line.len() {
                let group_type_size = text_group.group_type.len();
                if text_group.group_type[group_type_size - 1] == GroupType::Same {
                    text_group.group_type.push(GroupType::New);
                    text_group.data_group.push(new_group.clone());
                    new_group.clear();
                }
                new_group.extend_from_slice(&new_by_line[new_iter]);
                new_iter += 1;
                continue;
            }
            if new_iter >= new_by_line.len() {
                old_iter += 1;
                continue;
            }
            {
                ::std::io::_print(format_args!("old_iter: {0}\n", old_iter));
            };
            {
                ::std::io::_print(format_args!("new_iter: {0}\n", new_iter));
            };
            {
                ::std::io::_print(
                    format_args!("old line: {0:?}\n", old_by_line[old_iter]),
                );
            };
            {
                ::std::io::_print(
                    format_args!("new line: {0:?}\n", new_by_line[new_iter]),
                );
            };
            {
                ::std::io::_print(
                    format_args!("group_type: {0:?}\n", text_group.group_type),
                );
            };
            {
                ::std::io::_print(format_args!("new_group: {0:?}\n", new_group));
            };
            if old_by_line[old_iter] == new_by_line[new_iter] {
                {
                    ::std::io::_print(
                        format_args!(
                            "LAST GROUP TYPE: {0:?}\n",
                            text_group.group_type.last(),
                        ),
                    );
                };
                if first_line_flag {
                    new_group.extend_from_slice(&new_by_line[new_iter]);
                    text_group.group_type.push(GroupType::Same);
                    old_iter += 1;
                    new_iter += 1;
                    first_line_flag = false;
                    continue;
                }
                let group_type_size = text_group.group_type.len();
                if text_group.group_type[group_type_size - 1] == GroupType::Same {
                    new_group.extend_from_slice(&new_by_line[new_iter]);
                    old_iter += 1;
                    new_iter += 1;
                    continue;
                } else {
                    text_group.data_group.push(new_group.clone());
                    text_group.group_type.push(GroupType::Same);
                    new_group.clear();
                    new_group.extend_from_slice(&new_by_line[new_iter]);
                    old_iter += 1;
                    new_iter += 1;
                    text_group.group_breaks.push(new_iter);
                    continue;
                }
            }
            if old_by_line[old_iter] != new_by_line[new_iter] {
                if first_line_flag {
                    new_group.extend_from_slice(&new_by_line[new_iter]);
                    text_group.group_type.push(GroupType::New);
                    new_iter += 1;
                    first_line_flag = false;
                    continue;
                }
                let group_type_size = text_group.group_type.len();
                if text_group.group_type[group_type_size - 1] == GroupType::New {
                    new_group.extend_from_slice(&new_by_line[new_iter]);
                    new_iter += 1;
                    continue;
                } else {
                    text_group.data_group.push(new_group.clone());
                    text_group.group_type.push(GroupType::New);
                    new_group.clear();
                    new_group.extend_from_slice(&new_by_line[new_iter]);
                    new_iter += 1;
                    text_group.group_breaks.push(new_iter);
                    continue;
                }
            }
        }
        if !new_group.is_empty() {
            text_group.data_group.push(new_group.clone());
        }
        {
            ::std::io::_print(
                format_args!("text_grou is {0:?}\n", text_group.data_group),
            );
        };
        text_group
    }
    pub fn merge_file(
        old_file_group: &FileGrouped,
        new_file_group: &FileGrouped,
    ) -> Vec<u8> {
        let mut ret_vec: Vec<u8> = Vec::new();
        let mut old_it = 0;
        let mut new_it = 0;
        {
            ::std::io::_print(
                format_args!("old file group was {0:?}\n", old_file_group.data_group),
            );
        };
        {
            ::std::io::_print(
                format_args!("new file group was {0:?}\n", new_file_group.data_group),
            );
        };
        while old_it < old_file_group.data_group.len()
            || new_it < new_file_group.data_group.len()
        {
            if old_it >= old_file_group.data_group.len() {
                ret_vec.extend_from_slice(&new_file_group.data_group[new_it]);
                new_it += 1;
                continue;
            }
            if new_it >= new_file_group.data_group.len() {
                ret_vec.extend_from_slice(&old_file_group.data_group[old_it]);
                old_it += 1;
                continue;
            }
            if new_file_group.group_type[new_it] == GroupType::New {
                ret_vec.extend_from_slice(&new_file_group.data_group[new_it]);
                new_it += 1;
                continue;
            }
            if new_file_group.group_type[new_it] == GroupType::Same {
                if old_file_group.group_type[old_it] == GroupType::New {
                    ret_vec.extend_from_slice(&old_file_group.data_group[old_it]);
                    old_it += 1;
                } else if old_file_group.group_type[old_it] == GroupType::Same {
                    ret_vec.extend_from_slice(&new_file_group.data_group[new_it]);
                    old_it += 1;
                    new_it += 1;
                } else {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "merge found a delete and i haven\'t gotten there yet",
                            ),
                        );
                    };
                }
            }
        }
        ret_vec
    }
    pub fn regroup(file_group: &FileGrouped) -> Vec<u8> {
        let mut ret_vec = Vec::new();
        for ii in 0..file_group.data_group.len() {
            if file_group.group_type[ii] == GroupType::New {
                continue;
            }
            ret_vec.extend_from_slice(&file_group.data_group[ii]);
        }
        ret_vec
    }
}
use crate::merging::{split_by_changes, merge_file};
mod helpers {
    use fuser::{INodeNo, FileAttr, FileType};
    use sha2::{Sha256, Digest};
    use crate::HashMap;
    use crate::NodeKind;
    use crate::FileNode;
    use crate::datatypes::RustyCloud;
    pub fn unwrap_parent_dir(
        node_map: &mut HashMap<fuser::INodeNo, FileNode>,
        parent_ino: INodeNo,
    ) -> &mut HashMap<String, u64> {
        match node_map.get_mut(&parent_ino) {
            Some(parent_node) => {
                match &mut parent_node.file_type {
                    NodeKind::Directory { children } => children,
                    NodeKind::File { .. } => {
                        ::core::panicking::panic_fmt(
                            format_args!("Parent node was a file"),
                        );
                    }
                    NodeKind::Symlink { .. } => {
                        ::core::panicking::panic_fmt(
                            format_args!("Parent node was a symlink"),
                        );
                    }
                }
            }
            None => {
                ::core::panicking::panic_fmt(
                    format_args!("Tried unwrapping parent, failed"),
                );
            }
        }
    }
    pub fn new_inode_no(fs: &RustyCloud) -> u64 {
        let ino_num: u64;
        if fs.free_inode.lock().unwrap().is_empty() {
            {
                ::std::io::_print(format_args!("using new inode number\n"));
            };
            ino_num = *fs.next_inode.lock().unwrap();
            *fs.next_inode.lock().unwrap() += 1;
        } else {
            {
                ::std::io::_print(format_args!("using available Inode number\n"));
            };
            let new_inode_index = fs.free_inode.lock().unwrap().len() - 1;
            ino_num = fs.free_inode.lock().unwrap()[new_inode_index];
            fs.free_inode.lock().unwrap().pop();
        }
        ino_num
    }
    pub fn new_file_node(
        new_attr: &FileAttr,
        parent_path: &str,
        node_name: &str,
    ) -> Option<FileNode> {
        let mut file_path = parent_path.to_owned() + node_name;
        let new_f_node: FileNode;
        match new_attr.kind {
            FileType::Directory => {
                file_path = file_path.clone() + "/";
                new_f_node = FileNode {
                    attr: *new_attr,
                    file_type: NodeKind::Directory {
                        children: HashMap::new(),
                    },
                    path: file_path,
                    hash: None,
                };
            }
            FileType::RegularFile => {
                let data_vec = Vec::new();
                let hash = Sha256::digest(&data_vec);
                new_f_node = FileNode {
                    attr: *new_attr,
                    hash: Some(hash),
                    file_type: NodeKind::File {
                        file_data: data_vec,
                    },
                    path: file_path,
                };
            }
            FileType::Symlink => {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("symlinks are dealt with in function"),
                    );
                };
            }
            FileType::NamedPipe
            | FileType::CharDevice
            | FileType::BlockDevice
            | FileType::Socket => {
                {
                    ::std::io::_print(format_args!("ERROR, BAD THING HAPPENED\n"));
                };
                {
                    ::std::io::_print(
                        format_args!(
                            "NOT IMPLEMENTED FOR NAMED PIPES, CHAR DEVICES, etc\n",
                        ),
                    );
                };
                {
                    ::std::io::_print(format_args!("SEE new_file_node\n"));
                };
                return None;
            }
        }
        Some(new_f_node)
    }
}
use crate::helpers::{unwrap_parent_dir, new_inode_no, new_file_node};
mod datatypes {
    use fuser::{FileAttr, INodeNo};
    use sha2::Sha256;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicU64;
    use std::fmt;
    pub enum NodeKind {
        File { file_data: Vec<u8> },
        Directory { children: HashMap<String, u64> },
        Symlink { target_ino: String },
    }
    pub struct FileNode {
        pub attr: FileAttr,
        pub file_type: NodeKind,
        pub path: String,
        pub hash: Option<sha2::digest::Output<Sha256>>,
    }
    pub struct RustyCloud {
        pub f_node: Mutex<HashMap<fuser::INodeNo, FileNode>>,
        pub next_inode: Mutex<u64>,
        pub free_inode: Mutex<Vec<u64>>,
        pub next_fh: AtomicU64,
        pub splits: Mutex<HashMap<sha2::digest::Output<Sha256>, FileGrouped>>,
        pub atomic_ino: Mutex<INodeNo>,
        pub is_client: bool,
    }
    pub enum GroupType {
        Same,
        New,
        Deleted,
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for GroupType {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for GroupType {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            ::core::intrinsics::discriminant_value(self)
                == ::core::intrinsics::discriminant_value(other)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for GroupType {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    GroupType::Same => "Same",
                    GroupType::New => "New",
                    GroupType::Deleted => "Deleted",
                },
            )
        }
    }
    impl fmt::Display for GroupType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                GroupType::Same => f.write_fmt(format_args!("Grouping: Same")),
                GroupType::New => f.write_fmt(format_args!("Grouping: New")),
                GroupType::Deleted => f.write_fmt(format_args!("Grouping: Deleted")),
            }
        }
    }
    pub struct FileGrouped {
        pub data_group: Vec<Vec<u8>>,
        pub group_type: Vec<GroupType>,
        pub group_breaks: Vec<usize>,
        pub made_from: sha2::digest::Output<Sha256>,
    }
}
use crate::datatypes::{FileNode, NodeKind, FileGrouped, GroupType, RustyCloud};
const TIMEOUT: time::Duration = time::Duration::new(2, 0);
impl RustyCloud {
    pub fn new(is_client: bool) -> Self {
        let next_inode = Mutex::new(2);
        let free_inode = Mutex::new(Vec::new());
        let f_node = Mutex::new(HashMap::new());
        let splits = Mutex::new(HashMap::new());
        let atomic_ino = Mutex::new(INodeNo(0));
        let next_fh = AtomicU64::new(3);
        let root_attr = FileAttr {
            ino: INodeNo(1),
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 755,
            nlink: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };
        f_node
            .lock()
            .unwrap()
            .insert(
                fuser::INodeNo(1),
                FileNode {
                    attr: root_attr,
                    file_type: NodeKind::Directory {
                        children: HashMap::new(),
                    },
                    path: String::from("/"),
                    hash: None,
                },
            );
        Self {
            f_node,
            next_fh,
            next_inode,
            free_inode,
            splits,
            atomic_ino,
            is_client,
        }
    }
}
impl fmt::Display for RustyCloud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let root_node = INodeNo(1);
        let hash_m = self.f_node.lock().unwrap();
        let root_file_node = match hash_m.get(&root_node) {
            Some(r) => r,
            None => return f.write_fmt(format_args!("error displaying information")),
        };
        let inode = root_file_node.attr.ino;
        let size = root_file_node.attr.size;
        f.write_fmt(format_args!("inode: {0} \n size: {1}\n", inode, size))
    }
}
impl Filesystem for RustyCloud {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let name_str = name.to_str().unwrap();
        match children.get(name_str) {
            Some(inum) => {
                let inode = INodeNo(*inum);
                let file_node = node_map.get_mut(&inode).unwrap();
                match &file_node.file_type {
                    NodeKind::File { file_data } => {
                        file_node.attr.size = file_data.len() as u64;
                    }
                    NodeKind::Directory { .. } => {}
                    NodeKind::Symlink { .. } => {}
                };
                reply.entry(&TIMEOUT, &file_node.attr, Generation(0))
            }
            None => {
                {
                    ::std::io::_print(format_args!("no file matching\n"));
                };
                reply.error(Errno::ENOENT)
            }
        }
    }
    fn getattr(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: Option<FileHandle>,
        reply: ReplyAttr,
    ) {
        let mut node_map = self.f_node.lock().unwrap();
        let node = node_map.get_mut(&ino).unwrap();
        match &mut node.file_type {
            NodeKind::File { file_data } => {
                node.attr.size = file_data.len() as u64;
                reply.attr(&TIMEOUT, &node.attr)
            }
            NodeKind::Directory { .. } => reply.attr(&TIMEOUT, &node.attr),
            NodeKind::Symlink { .. } => reply.attr(&TIMEOUT, &node.attr),
        }
    }
    fn setattr(
        &self,
        _req: &Request,
        ino: INodeNo,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        _attime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        _fh: Option<FileHandle>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<BsdFileFlags>,
        reply: ReplyAttr,
    ) {
        let mut hash_m = self.f_node.lock().unwrap();
        let node = hash_m.get_mut(&ino).unwrap();
        match size {
            Some(re) => {
                node.attr.size = re;
                match &mut node.file_type {
                    NodeKind::File { file_data } => {
                        file_data.resize(re as usize, 0);
                        reply.attr(&TIMEOUT, &node.attr)
                    }
                    NodeKind::Directory { .. } => reply.attr(&TIMEOUT, &node.attr),
                    NodeKind::Symlink { .. } => reply.attr(&TIMEOUT, &node.attr),
                }
            }
            None => reply.attr(&TIMEOUT, &node.attr),
        }
    }
    fn create(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        {
            ::std::io::_print(format_args!("creating a file\n"));
        };
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let str_name = name.to_str().unwrap();
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
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 755,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };
        let p_result = node_map.get(&parent).unwrap();
        let new_f_node: FileNode = new_file_node(
                &new_file_attr,
                &p_result.path.clone(),
                &str_ret,
            )
            .unwrap();
        node_map.insert(INodeNo(ino_num), new_f_node);
        let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
        reply
            .created(
                &TIMEOUT,
                &new_file_attr,
                Generation(0),
                FileHandle(fh),
                FopenFlags::empty(),
            )
    }
    fn mkdir(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let str_name = name.to_str().unwrap();
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
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 755,
            nlink: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };
        let p_result = node_map.get(&parent).unwrap();
        let new_d_node: FileNode = new_file_node(
                &new_directory_attr,
                &p_result.path.clone(),
                &str_ret,
            )
            .unwrap();
        node_map.insert(INodeNo(ino_num), new_d_node);
        reply.entry(&TIMEOUT, &new_directory_attr, Generation(0));
    }
    fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let name_string = String::from(name.to_str().unwrap());
        let ino_num = children.remove(&name_string).unwrap();
        node_map.remove(&INodeNo(ino_num));
        self.free_inode.lock().unwrap().push(ino_num);
        reply.ok();
    }
    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        {
            ::std::io::_print(format_args!("unlinking a file\n"));
        };
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let name_string = String::from(name.to_str().unwrap());
        let ino_num = children.remove(&name_string).unwrap();
        let file_node = node_map.get_mut(&INodeNo(ino_num)).unwrap();
        if file_node.attr.nlink > 1 {
            file_node.attr.nlink -= 1;
        } else {
            node_map.remove(&INodeNo(ino_num));
            self.free_inode.lock().unwrap().push(ino_num);
        }
        reply.ok();
    }
    fn rename(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        newparent: INodeNo,
        newname: &OsStr,
        _flags: RenameFlags,
        reply: ReplyEmpty,
    ) {
        {
            ::std::io::_print(format_args!("rename of a file is happening\n"));
        };
        {
            ::std::io::_print(format_args!("new name is {0:?}\n", newname));
        };
        let mut node_map = self.f_node.lock().unwrap();
        let mut a_ino = self.atomic_ino.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, parent);
        let old_name: String = String::from(name.to_str().unwrap());
        let new_name: String = String::from(newname.to_str().unwrap());
        match children.get_mut(&old_name) {
            Some(_) => {
                let ino: u64 = children.remove(&old_name).unwrap();
                *a_ino = INodeNo(ino);
                if parent == newparent {
                    children.insert(new_name, ino);
                    reply.ok()
                } else {
                    let new_children = unwrap_parent_dir(&mut node_map, newparent);
                    new_children.insert(new_name, ino);
                    reply.ok();
                }
                match node_map.get_mut(&INodeNo(ino)) {
                    Some(node) => {
                        while node.path.pop().unwrap() != '/' {}
                        node.path.push('/');
                        node.path.insert_str(node.path.len(), newname.to_str().unwrap());
                    }
                    None => {
                        {
                            ::std::io::_print(format_args!("should never happen\n"));
                        };
                    }
                }
            }
            None => {
                {
                    ::std::io::_print(format_args!("could not find file to rename\n"));
                };
                reply.error(Errno::ENOENT)
            }
        }
    }
    fn open(&self, _req: &Request, _ino: INodeNo, _flags: OpenFlags, reply: ReplyOpen) {
        let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
        reply.opened(FileHandle(fh), FopenFlags::empty());
    }
    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        _offset: u64,
        _size: u32,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyData,
    ) {
        let node_map = self.f_node.lock().unwrap();
        let file_node = node_map.get(&ino).unwrap();
        match &file_node.file_type {
            NodeKind::File { file_data } => {
                reply.data(file_data);
            }
            NodeKind::Directory { .. } => {
                {
                    ::std::io::_print(format_args!("read error, reading a dir\n"));
                };
                reply.error(Errno::ENOENT);
            }
            NodeKind::Symlink { .. } => {
                {
                    ::std::io::_print(
                        format_args!("TODO: read the symlink parent data\n"),
                    );
                };
            }
        };
    }
    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        let mut node_map = self.f_node.lock().unwrap();
        let children = unwrap_parent_dir(&mut node_map, ino);
        let child_files = children.clone();
        for (i, (name, ino)) in child_files.iter().enumerate().skip(offset as usize) {
            let file_node = node_map.get(&INodeNo(*ino)).unwrap();
            let ft = match file_node.file_type {
                NodeKind::Directory { .. } => FileType::Directory,
                NodeKind::File { .. } => FileType::RegularFile,
                NodeKind::Symlink { .. } => FileType::Symlink,
            };
            let buffer_full = reply.add(INodeNo(*ino), (i + 1) as u64, ft, name);
            if buffer_full {
                break;
            }
        }
        reply.ok();
    }
    fn write(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        _offset: u64,
        data: &[u8],
        _write_flags: WriteFlags,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyWrite,
    ) {
        {
            ::std::io::_print(format_args!("write flags are {0:?}\n", _write_flags));
        };
        let mut node_map = self.f_node.lock().unwrap();
        let mut splits = self.splits.lock().unwrap();
        let atomic_ino = self.atomic_ino.lock().unwrap();
        let mut atomic_path = String::new();
        let mut atomic_hash: Option<sha2::digest::Output<Sha256>> = None;
        match node_map.get_mut(&atomic_ino) {
            Some(node) => {
                {
                    ::std::io::_print(format_args!("path of the found inode was \n"));
                };
                {
                    ::std::io::_print(format_args!("{0:?}\n", node.path));
                };
                node.path.pop().unwrap();
                atomic_path = node.path.clone();
                atomic_hash = node.hash;
            }
            None => {
                {
                    ::std::io::_print(format_args!("did not find an atomic inode\n"));
                };
            }
        }
        match node_map.get_mut(&ino) {
            Some(node) => {
                match &mut node.file_type {
                    NodeKind::File { file_data } => {
                        let mut old_hash = node.hash.unwrap();
                        {
                            ::std::io::_print(
                                format_args!("looking for hash {0:?}\n", old_hash),
                            );
                        };
                        let file_hash = Sha256::digest(data);
                        if file_hash == old_hash {
                            reply.written(data.len() as u32);
                            return;
                        }
                        if node.path == atomic_path {
                            {
                                ::std::io::_print(format_args!("atomic file was saved\n"));
                            };
                            old_hash = atomic_hash.unwrap();
                        }
                        match splits.get_mut(&old_hash) {
                            Some(old_file_group) => {
                                let mut new_file_group: FileGrouped = split_by_changes(
                                    old_file_group,
                                    data.to_vec(),
                                    file_hash,
                                );
                                let new_file_data: Vec<u8> = merge_file(
                                    old_file_group,
                                    &new_file_group,
                                );
                                for ii in 0..old_file_group.group_type.len() {
                                    for yy in 0..new_file_group.group_type.len() {
                                        if old_file_group.data_group[ii]
                                            == new_file_group.data_group[yy]
                                        {
                                            new_file_group.group_type[yy] = GroupType::Same;
                                        }
                                    }
                                }
                                splits.remove(&old_hash);
                                {
                                    ::std::io::_print(
                                        format_args!(
                                            "inserting hash into splits {0:?}\n",
                                            file_hash,
                                        ),
                                    );
                                };
                                splits.insert(file_hash, new_file_group);
                                node.hash = Some(file_hash);
                                for ch in new_file_data.iter() {
                                    file_data.push(*ch);
                                }
                                reply.written(data.len() as u32);
                            }
                            None => {
                                {
                                    ::std::io::_print(
                                        format_args!("was looking for hash {0:?}\n", old_hash),
                                    );
                                };
                                {
                                    ::std::io::_print(
                                        format_args!("assuming first file write\n"),
                                    );
                                };
                                let base_data = ::alloc::boxed::box_assume_init_into_vec_unsafe(
                                    ::alloc::intrinsics::write_box_via_move(
                                        ::alloc::boxed::Box::new_uninit(),
                                        [data.to_vec()],
                                    ),
                                );
                                let group_types = ::alloc::boxed::box_assume_init_into_vec_unsafe(
                                    ::alloc::intrinsics::write_box_via_move(
                                        ::alloc::boxed::Box::new_uninit(),
                                        [GroupType::Same],
                                    ),
                                );
                                let base_group = FileGrouped {
                                    data_group: base_data,
                                    group_type: group_types,
                                    group_breaks: Vec::new(),
                                    made_from: file_hash,
                                };
                                splits.insert(file_hash, base_group);
                                {
                                    ::std::io::_print(
                                        format_args!("instered {0:?} into splits\n", file_hash),
                                    );
                                };
                                for ch in data.iter() {
                                    file_data.push(*ch);
                                }
                                node.hash = Some(file_hash);
                                reply.written(data.len() as u32);
                            }
                        }
                    }
                    NodeKind::Directory { .. } => {
                        {
                            ::std::io::_print(
                                format_args!("WRITE: can\'t write to a dir\n"),
                            );
                        };
                        reply.error(Errno::ENOENT);
                    }
                    NodeKind::Symlink { .. } => {
                        {
                            ::std::io::_print(
                                format_args!("WRITE: can\'t write to symlink\n"),
                            );
                        };
                        reply.error(Errno::ENOENT)
                    }
                }
            }
            None => {
                {
                    ::std::io::_print(
                        format_args!("write error, could not find file in system\n"),
                    );
                };
                reply.error(Errno::ENOENT);
            }
        };
    }
    fn link(
        &self,
        _req: &Request,
        ino: INodeNo,
        newparent: INodeNo,
        newname: &OsStr,
        reply: ReplyEntry,
    ) {
        let mut node_map = self.f_node.lock().unwrap();
        let file_node = node_map.get_mut(&ino).unwrap();
        file_node.attr.nlink += 1;
        let attr_copy = file_node.attr;
        if #[allow(non_exhaustive_omitted_patterns)]
        match file_node.file_type {
            NodeKind::File { .. } => true,
            _ => false,
        } {
            let name_str = newname.to_str().unwrap();
            let ret_str = String::from(name_str);
            let children = unwrap_parent_dir(&mut node_map, newparent);
            children.insert(ret_str, ino.0);
            reply.entry(&TIMEOUT, &attr_copy, Generation(0));
        } else {
            {
                ::std::io::_print(format_args!("hard link error, not a file\n"));
            };
            reply.error(Errno::ENOENT);
        }
    }
    fn symlink(
        &self,
        _req: &Request,
        parent: INodeNo,
        link_name: &OsStr,
        target: &Path,
        reply: ReplyEntry,
    ) {
        let mut node_map = self.f_node.lock().unwrap();
        let sym_ino: u64 = new_inode_no(self);
        let children = unwrap_parent_dir(&mut node_map, parent);
        let sym_str: &str = link_name.to_str().unwrap();
        children.insert(String::from(sym_str), sym_ino);
        let sym_attr = FileAttr {
            ino: INodeNo(sym_ino),
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Symlink,
            perm: 755,
            nlink: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };
        let p_node = node_map.get(&parent).unwrap();
        let sym_node = FileNode {
            attr: sym_attr,
            file_type: NodeKind::Symlink {
                target_ino: String::from(target.to_str().unwrap()),
            },
            path: p_node.path.clone() + sym_str,
            hash: None,
        };
        node_map.insert(INodeNo(sym_ino), sym_node);
        reply.entry(&TIMEOUT, &sym_attr, Generation(0));
    }
    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
        let node_map = self.f_node.lock().unwrap();
        let link_node = &node_map.get(&ino).unwrap().file_type;
        if let NodeKind::Symlink { target_ino } = link_node {
            reply.data(target_ino.as_bytes());
        } else {
            {
                ::std::io::_print(
                    format_args!("tried reading non symlink as symlink\n"),
                );
            };
            reply.error(Errno::ENOENT)
        }
    }
    fn release(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _fh: FileHandle,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        reply.ok()
    }
}
use tonic::transport::Server;
pub mod rusty_net {
    #[allow(clippy::all)]
    pub struct LookupReq {
        #[prost(uint64, tag = "1")]
        pub parent_ino: u64,
        #[prost(string, tag = "2")]
        pub file_name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for LookupReq {
        #[inline]
        fn clone(&self) -> Self {
            LookupReq {
                parent_ino: ::core::clone::Clone::clone(&self.parent_ino),
                file_name: ::core::clone::Clone::clone(&self.file_name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for LookupReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for LookupReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.parent_ino == other.parent_ino && self.file_name == other.file_name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for LookupReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for LookupReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.parent_ino, state);
            ::core::hash::Hash::hash(&self.file_name, state)
        }
    }
    impl ::prost::Message for LookupReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.parent_ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.parent_ino, buf);
            }
            if self.file_name != "" {
                ::prost::encoding::string::encode(2u32, &self.file_name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "LookupReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.parent_ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "parent_ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.file_name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "file_name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.parent_ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.parent_ino)
                } else {
                    0
                }
                + if self.file_name != "" {
                    ::prost::encoding::string::encoded_len(2u32, &self.file_name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.parent_ino = 0u64;
            self.file_name.clear();
        }
    }
    impl ::core::default::Default for LookupReq {
        fn default() -> Self {
            LookupReq {
                parent_ino: 0u64,
                file_name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for LookupReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("LookupReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.parent_ino)
                };
                builder.field("parent_ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.file_name)
                };
                builder.field("file_name", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct FileAttr {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(uint64, tag = "2")]
        pub perm: u64,
        #[prost(uint64, tag = "3")]
        pub size: u64,
        #[prost(uint64, tag = "4")]
        pub blocks: u64,
        #[prost(uint64, tag = "5")]
        pub uid: u64,
        /// might need more?
        #[prost(uint64, tag = "6")]
        pub gid: u64,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for FileAttr {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for FileAttr {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for FileAttr {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for FileAttr {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for FileAttr {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.perm == other.perm && self.size == other.size
                && self.blocks == other.blocks && self.uid == other.uid
                && self.gid == other.gid
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for FileAttr {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for FileAttr {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.perm, state);
            ::core::hash::Hash::hash(&self.size, state);
            ::core::hash::Hash::hash(&self.blocks, state);
            ::core::hash::Hash::hash(&self.uid, state);
            ::core::hash::Hash::hash(&self.gid, state)
        }
    }
    impl ::prost::Message for FileAttr {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.perm != 0u64 {
                ::prost::encoding::uint64::encode(2u32, &self.perm, buf);
            }
            if self.size != 0u64 {
                ::prost::encoding::uint64::encode(3u32, &self.size, buf);
            }
            if self.blocks != 0u64 {
                ::prost::encoding::uint64::encode(4u32, &self.blocks, buf);
            }
            if self.uid != 0u64 {
                ::prost::encoding::uint64::encode(5u32, &self.uid, buf);
            }
            if self.gid != 0u64 {
                ::prost::encoding::uint64::encode(6u32, &self.gid, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "FileAttr";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.perm;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "perm");
                            error
                        })
                }
                3u32 => {
                    let mut value = &mut self.size;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "size");
                            error
                        })
                }
                4u32 => {
                    let mut value = &mut self.blocks;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "blocks");
                            error
                        })
                }
                5u32 => {
                    let mut value = &mut self.uid;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "uid");
                            error
                        })
                }
                6u32 => {
                    let mut value = &mut self.gid;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "gid");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.perm != 0u64 {
                    ::prost::encoding::uint64::encoded_len(2u32, &self.perm)
                } else {
                    0
                }
                + if self.size != 0u64 {
                    ::prost::encoding::uint64::encoded_len(3u32, &self.size)
                } else {
                    0
                }
                + if self.blocks != 0u64 {
                    ::prost::encoding::uint64::encoded_len(4u32, &self.blocks)
                } else {
                    0
                }
                + if self.uid != 0u64 {
                    ::prost::encoding::uint64::encoded_len(5u32, &self.uid)
                } else {
                    0
                }
                + if self.gid != 0u64 {
                    ::prost::encoding::uint64::encoded_len(6u32, &self.gid)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.perm = 0u64;
            self.size = 0u64;
            self.blocks = 0u64;
            self.uid = 0u64;
            self.gid = 0u64;
        }
    }
    impl ::core::default::Default for FileAttr {
        fn default() -> Self {
            FileAttr {
                ino: 0u64,
                perm: 0u64,
                size: 0u64,
                blocks: 0u64,
                uid: 0u64,
                gid: 0u64,
            }
        }
    }
    impl ::core::fmt::Debug for FileAttr {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("FileAttr");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.perm)
                };
                builder.field("perm", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.size)
                };
                builder.field("size", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.blocks)
                };
                builder.field("blocks", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.uid)
                };
                builder.field("uid", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.gid)
                };
                builder.field("gid", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct LookupRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for LookupRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for LookupRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<::core::option::Option<FileAttr>>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for LookupRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for LookupRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for LookupRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for LookupRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for LookupRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state)
        }
    }
    impl ::prost::Message for LookupRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "LookupRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
        }
    }
    impl ::core::default::Default for LookupRes {
        fn default() -> Self {
            LookupRes {
                found: false,
                attr: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for LookupRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("LookupRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct GetattrReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for GetattrReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for GetattrReq {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for GetattrReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for GetattrReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for GetattrReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for GetattrReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for GetattrReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state)
        }
    }
    impl ::prost::Message for GetattrReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "GetattrReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
        }
    }
    impl ::core::default::Default for GetattrReq {
        fn default() -> Self {
            GetattrReq { ino: 0u64 }
        }
    }
    impl ::core::fmt::Debug for GetattrReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("GetattrReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct GetattrRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for GetattrRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for GetattrRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<::core::option::Option<FileAttr>>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for GetattrRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for GetattrRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for GetattrRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for GetattrRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for GetattrRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state)
        }
    }
    impl ::prost::Message for GetattrRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "GetattrRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
        }
    }
    impl ::core::default::Default for GetattrRes {
        fn default() -> Self {
            GetattrRes {
                found: false,
                attr: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for GetattrRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("GetattrRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct SetattrReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(uint64, repeated, tag = "2")]
        pub size: ::prost::alloc::vec::Vec<u64>,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for SetattrReq {
        #[inline]
        fn clone(&self) -> Self {
            SetattrReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                size: ::core::clone::Clone::clone(&self.size),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for SetattrReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for SetattrReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.size == other.size
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for SetattrReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::vec::Vec<u64>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for SetattrReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.size, state)
        }
    }
    impl ::prost::Message for SetattrReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            ::prost::encoding::uint64::encode_packed(2u32, &self.size, buf);
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "SetattrReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.size;
                    ::prost::encoding::uint64::merge_repeated(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "size");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                } + ::prost::encoding::uint64::encoded_len_packed(2u32, &self.size)
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.size.clear();
        }
    }
    impl ::core::default::Default for SetattrReq {
        fn default() -> Self {
            SetattrReq {
                ino: 0u64,
                size: ::prost::alloc::vec::Vec::new(),
            }
        }
    }
    impl ::core::fmt::Debug for SetattrReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("SetattrReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    struct ScalarWrapper<'a>(&'a ::prost::alloc::vec::Vec<u64>);
                    impl<'a> ::core::fmt::Debug for ScalarWrapper<'a> {
                        fn fmt(
                            &self,
                            f: &mut ::core::fmt::Formatter,
                        ) -> ::core::fmt::Result {
                            let mut vec_builder = f.debug_list();
                            for v in self.0 {
                                #[allow(non_snake_case)]
                                fn Inner<T>(v: T) -> T {
                                    v
                                }
                                vec_builder.entry(&Inner(v));
                            }
                            vec_builder.finish()
                        }
                    }
                    ScalarWrapper(&self.size)
                };
                builder.field("size", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct SetattrRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for SetattrRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for SetattrRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<::core::option::Option<FileAttr>>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for SetattrRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for SetattrRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for SetattrRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for SetattrRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for SetattrRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state)
        }
    }
    impl ::prost::Message for SetattrRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "SetattrRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
        }
    }
    impl ::core::default::Default for SetattrRes {
        fn default() -> Self {
            SetattrRes {
                found: false,
                attr: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for SetattrRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("SetattrRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct CreateReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for CreateReq {
        #[inline]
        fn clone(&self) -> Self {
            CreateReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for CreateReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for CreateReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.name == other.name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for CreateReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for CreateReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.name, state)
        }
    }
    impl ::prost::Message for CreateReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(2u32, &self.name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "CreateReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(2u32, &self.name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.name.clear();
        }
    }
    impl ::core::default::Default for CreateReq {
        fn default() -> Self {
            CreateReq {
                ino: 0u64,
                name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for CreateReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("CreateReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            builder.finish()
        }
    }
    /// TODO: may have to fix this
    #[allow(clippy::all)]
    pub struct CreateRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
        #[prost(string, tag = "3")]
        pub result_path: ::prost::alloc::string::String,
        #[prost(string, tag = "4")]
        pub str_ret: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for CreateRes {
        #[inline]
        fn clone(&self) -> Self {
            CreateRes {
                found: ::core::clone::Clone::clone(&self.found),
                attr: ::core::clone::Clone::clone(&self.attr),
                result_path: ::core::clone::Clone::clone(&self.result_path),
                str_ret: ::core::clone::Clone::clone(&self.str_ret),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for CreateRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for CreateRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
                && self.result_path == other.result_path && self.str_ret == other.str_ret
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for CreateRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for CreateRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state);
            ::core::hash::Hash::hash(&self.result_path, state);
            ::core::hash::Hash::hash(&self.str_ret, state)
        }
    }
    impl ::prost::Message for CreateRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
            if self.result_path != "" {
                ::prost::encoding::string::encode(3u32, &self.result_path, buf);
            }
            if self.str_ret != "" {
                ::prost::encoding::string::encode(4u32, &self.str_ret, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "CreateRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                3u32 => {
                    let mut value = &mut self.result_path;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "result_path");
                            error
                        })
                }
                4u32 => {
                    let mut value = &mut self.str_ret;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "str_ret");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
                + if self.result_path != "" {
                    ::prost::encoding::string::encoded_len(3u32, &self.result_path)
                } else {
                    0
                }
                + if self.str_ret != "" {
                    ::prost::encoding::string::encoded_len(4u32, &self.str_ret)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
            self.result_path.clear();
            self.str_ret.clear();
        }
    }
    impl ::core::default::Default for CreateRes {
        fn default() -> Self {
            CreateRes {
                found: false,
                attr: ::core::default::Default::default(),
                result_path: ::prost::alloc::string::String::new(),
                str_ret: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for CreateRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("CreateRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.result_path)
                };
                builder.field("result_path", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.str_ret)
                };
                builder.field("str_ret", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct MkdirReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for MkdirReq {
        #[inline]
        fn clone(&self) -> Self {
            MkdirReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for MkdirReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for MkdirReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.name == other.name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for MkdirReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for MkdirReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.name, state)
        }
    }
    impl ::prost::Message for MkdirReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(2u32, &self.name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "MkdirReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(2u32, &self.name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.name.clear();
        }
    }
    impl ::core::default::Default for MkdirReq {
        fn default() -> Self {
            MkdirReq {
                ino: 0u64,
                name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for MkdirReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("MkdirReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct MkdirRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for MkdirRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for MkdirRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<::core::option::Option<FileAttr>>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for MkdirRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for MkdirRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for MkdirRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for MkdirRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for MkdirRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state)
        }
    }
    impl ::prost::Message for MkdirRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "MkdirRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
        }
    }
    impl ::core::default::Default for MkdirRes {
        fn default() -> Self {
            MkdirRes {
                found: false,
                attr: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for MkdirRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("MkdirRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct RmdirReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for RmdirReq {
        #[inline]
        fn clone(&self) -> Self {
            RmdirReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for RmdirReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for RmdirReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.name == other.name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for RmdirReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for RmdirReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.name, state)
        }
    }
    impl ::prost::Message for RmdirReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(2u32, &self.name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "RmdirReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(2u32, &self.name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.name.clear();
        }
    }
    impl ::core::default::Default for RmdirReq {
        fn default() -> Self {
            RmdirReq {
                ino: 0u64,
                name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for RmdirReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("RmdirReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct RmdirRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for RmdirRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for RmdirRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<::core::option::Option<FileAttr>>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for RmdirRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for RmdirRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for RmdirRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for RmdirRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for RmdirRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state)
        }
    }
    impl ::prost::Message for RmdirRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "RmdirRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
        }
    }
    impl ::core::default::Default for RmdirRes {
        fn default() -> Self {
            RmdirRes {
                found: false,
                attr: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for RmdirRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("RmdirRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct UnlinkReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for UnlinkReq {
        #[inline]
        fn clone(&self) -> Self {
            UnlinkReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for UnlinkReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for UnlinkReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.name == other.name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for UnlinkReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for UnlinkReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.name, state)
        }
    }
    impl ::prost::Message for UnlinkReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(2u32, &self.name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "UnlinkReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(2u32, &self.name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.name.clear();
        }
    }
    impl ::core::default::Default for UnlinkReq {
        fn default() -> Self {
            UnlinkReq {
                ino: 0u64,
                name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for UnlinkReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("UnlinkReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct UnlinkRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for UnlinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for UnlinkRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for UnlinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for UnlinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for UnlinkRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for UnlinkRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for UnlinkRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state)
        }
    }
    impl ::prost::Message for UnlinkRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "UnlinkRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.found = false;
        }
    }
    impl ::core::default::Default for UnlinkRes {
        fn default() -> Self {
            UnlinkRes { found: false }
        }
    }
    impl ::core::fmt::Debug for UnlinkRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("UnlinkRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct RenameReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
        #[prost(uint64, tag = "3")]
        pub new_ino: u64,
        #[prost(string, tag = "4")]
        pub new_name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for RenameReq {
        #[inline]
        fn clone(&self) -> Self {
            RenameReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                name: ::core::clone::Clone::clone(&self.name),
                new_ino: ::core::clone::Clone::clone(&self.new_ino),
                new_name: ::core::clone::Clone::clone(&self.new_name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for RenameReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for RenameReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.new_ino == other.new_ino
                && self.name == other.name && self.new_name == other.new_name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for RenameReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for RenameReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.name, state);
            ::core::hash::Hash::hash(&self.new_ino, state);
            ::core::hash::Hash::hash(&self.new_name, state)
        }
    }
    impl ::prost::Message for RenameReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(2u32, &self.name, buf);
            }
            if self.new_ino != 0u64 {
                ::prost::encoding::uint64::encode(3u32, &self.new_ino, buf);
            }
            if self.new_name != "" {
                ::prost::encoding::string::encode(4u32, &self.new_name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "RenameReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                3u32 => {
                    let mut value = &mut self.new_ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "new_ino");
                            error
                        })
                }
                4u32 => {
                    let mut value = &mut self.new_name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "new_name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(2u32, &self.name)
                } else {
                    0
                }
                + if self.new_ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(3u32, &self.new_ino)
                } else {
                    0
                }
                + if self.new_name != "" {
                    ::prost::encoding::string::encoded_len(4u32, &self.new_name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.name.clear();
            self.new_ino = 0u64;
            self.new_name.clear();
        }
    }
    impl ::core::default::Default for RenameReq {
        fn default() -> Self {
            RenameReq {
                ino: 0u64,
                name: ::prost::alloc::string::String::new(),
                new_ino: 0u64,
                new_name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for RenameReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("RenameReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.new_ino)
                };
                builder.field("new_ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.new_name)
                };
                builder.field("new_name", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct RenameRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for RenameRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for RenameRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for RenameRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for RenameRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for RenameRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for RenameRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for RenameRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state)
        }
    }
    impl ::prost::Message for RenameRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "RenameRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.found = false;
        }
    }
    impl ::core::default::Default for RenameRes {
        fn default() -> Self {
            RenameRes { found: false }
        }
    }
    impl ::core::fmt::Debug for RenameRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("RenameRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct ReadReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for ReadReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReadReq {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for ReadReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReadReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReadReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for ReadReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for ReadReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state)
        }
    }
    impl ::prost::Message for ReadReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReadReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
        }
    }
    impl ::core::default::Default for ReadReq {
        fn default() -> Self {
            ReadReq { ino: 0u64 }
        }
    }
    impl ::core::fmt::Debug for ReadReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReadReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct ReadRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(bytes = "vec", tag = "2")]
        pub data: ::prost::alloc::vec::Vec<u8>,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReadRes {
        #[inline]
        fn clone(&self) -> Self {
            ReadRes {
                found: ::core::clone::Clone::clone(&self.found),
                data: ::core::clone::Clone::clone(&self.data),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReadRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReadRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.data == other.data
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for ReadRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::vec::Vec<u8>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for ReadRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.data, state)
        }
    }
    impl ::prost::Message for ReadRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if self.data != b"" as &[u8] {
                ::prost::encoding::bytes::encode(2u32, &self.data, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReadRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.data;
                    ::prost::encoding::bytes::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "data");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + if self.data != b"" as &[u8] {
                    ::prost::encoding::bytes::encoded_len(2u32, &self.data)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.found = false;
            self.data.clear();
        }
    }
    impl ::core::default::Default for ReadRes {
        fn default() -> Self {
            ReadRes {
                found: false,
                data: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for ReadRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReadRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.data)
                };
                builder.field("data", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct ReaddirReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(uint64, tag = "2")]
        pub offset: u64,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for ReaddirReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReaddirReq {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for ReaddirReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReaddirReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReaddirReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.offset == other.offset
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for ReaddirReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for ReaddirReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.offset, state)
        }
    }
    impl ::prost::Message for ReaddirReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.offset != 0u64 {
                ::prost::encoding::uint64::encode(2u32, &self.offset, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReaddirReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.offset;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "offset");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.offset != 0u64 {
                    ::prost::encoding::uint64::encoded_len(2u32, &self.offset)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.offset = 0u64;
        }
    }
    impl ::core::default::Default for ReaddirReq {
        fn default() -> Self {
            ReaddirReq {
                ino: 0u64,
                offset: 0u64,
            }
        }
    }
    impl ::core::fmt::Debug for ReaddirReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReaddirReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.offset)
                };
                builder.field("offset", &wrapper)
            };
            builder.finish()
        }
    }
    /// Filetype will be 0, 1, 2 relating
    /// 0: Directory
    /// 1: file
    /// 2: symlink
    #[allow(clippy::all)]
    pub struct DirEntry {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(uint64, tag = "2")]
        pub offset: u64,
        #[prost(uint32, tag = "3")]
        pub file_type: u32,
        #[prost(string, tag = "4")]
        pub name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for DirEntry {
        #[inline]
        fn clone(&self) -> Self {
            DirEntry {
                ino: ::core::clone::Clone::clone(&self.ino),
                offset: ::core::clone::Clone::clone(&self.offset),
                file_type: ::core::clone::Clone::clone(&self.file_type),
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for DirEntry {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for DirEntry {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.offset == other.offset
                && self.file_type == other.file_type && self.name == other.name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for DirEntry {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<u32>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for DirEntry {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.offset, state);
            ::core::hash::Hash::hash(&self.file_type, state);
            ::core::hash::Hash::hash(&self.name, state)
        }
    }
    impl ::prost::Message for DirEntry {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.offset != 0u64 {
                ::prost::encoding::uint64::encode(2u32, &self.offset, buf);
            }
            if self.file_type != 0u32 {
                ::prost::encoding::uint32::encode(3u32, &self.file_type, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(4u32, &self.name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "DirEntry";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.offset;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "offset");
                            error
                        })
                }
                3u32 => {
                    let mut value = &mut self.file_type;
                    ::prost::encoding::uint32::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "file_type");
                            error
                        })
                }
                4u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.offset != 0u64 {
                    ::prost::encoding::uint64::encoded_len(2u32, &self.offset)
                } else {
                    0
                }
                + if self.file_type != 0u32 {
                    ::prost::encoding::uint32::encoded_len(3u32, &self.file_type)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(4u32, &self.name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.offset = 0u64;
            self.file_type = 0u32;
            self.name.clear();
        }
    }
    impl ::core::default::Default for DirEntry {
        fn default() -> Self {
            DirEntry {
                ino: 0u64,
                offset: 0u64,
                file_type: 0u32,
                name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for DirEntry {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("DirEntry");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.offset)
                };
                builder.field("offset", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.file_type)
                };
                builder.field("file_type", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            builder.finish()
        }
    }
    /// TODO: I think this will be fine since the function ends with ok
    #[allow(clippy::all)]
    pub struct ReaddirRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, repeated, tag = "2")]
        pub dir_entries: ::prost::alloc::vec::Vec<DirEntry>,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReaddirRes {
        #[inline]
        fn clone(&self) -> Self {
            ReaddirRes {
                found: ::core::clone::Clone::clone(&self.found),
                dir_entries: ::core::clone::Clone::clone(&self.dir_entries),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReaddirRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReaddirRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.dir_entries == other.dir_entries
        }
    }
    impl ::prost::Message for ReaddirRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            for msg in &self.dir_entries {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReaddirRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.dir_entries;
                    ::prost::encoding::message::merge_repeated(
                            wire_type,
                            value,
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "dir_entries");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + ::prost::encoding::message::encoded_len_repeated(
                    2u32,
                    &self.dir_entries,
                )
        }
        fn clear(&mut self) {
            self.found = false;
            self.dir_entries.clear();
        }
    }
    impl ::core::default::Default for ReaddirRes {
        fn default() -> Self {
            ReaddirRes {
                found: false,
                dir_entries: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for ReaddirRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReaddirRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.dir_entries;
                builder.field("dir_entries", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct WriteReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(bytes = "vec", tag = "2")]
        pub data: ::prost::alloc::vec::Vec<u8>,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for WriteReq {
        #[inline]
        fn clone(&self) -> Self {
            WriteReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                data: ::core::clone::Clone::clone(&self.data),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for WriteReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for WriteReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.data == other.data
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for WriteReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::vec::Vec<u8>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for WriteReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.data, state)
        }
    }
    impl ::prost::Message for WriteReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.data != b"" as &[u8] {
                ::prost::encoding::bytes::encode(2u32, &self.data, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "WriteReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.data;
                    ::prost::encoding::bytes::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "data");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.data != b"" as &[u8] {
                    ::prost::encoding::bytes::encoded_len(2u32, &self.data)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.data.clear();
        }
    }
    impl ::core::default::Default for WriteReq {
        fn default() -> Self {
            WriteReq {
                ino: 0u64,
                data: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for WriteReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("WriteReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.data)
                };
                builder.field("data", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct WriteRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(uint64, tag = "2")]
        pub len: u64,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for WriteRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for WriteRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for WriteRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for WriteRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for WriteRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.len == other.len
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for WriteRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for WriteRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.len, state)
        }
    }
    impl ::prost::Message for WriteRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if self.len != 0u64 {
                ::prost::encoding::uint64::encode(2u32, &self.len, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "WriteRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.len;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "len");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + if self.len != 0u64 {
                    ::prost::encoding::uint64::encoded_len(2u32, &self.len)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.found = false;
            self.len = 0u64;
        }
    }
    impl ::core::default::Default for WriteRes {
        fn default() -> Self {
            WriteRes {
                found: false,
                len: 0u64,
            }
        }
    }
    impl ::core::fmt::Debug for WriteRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("WriteRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.len)
                };
                builder.field("len", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct LinkReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(uint64, tag = "2")]
        pub new_ino: u64,
        #[prost(string, tag = "3")]
        pub name: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for LinkReq {
        #[inline]
        fn clone(&self) -> Self {
            LinkReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                new_ino: ::core::clone::Clone::clone(&self.new_ino),
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for LinkReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for LinkReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.new_ino == other.new_ino
                && self.name == other.name
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for LinkReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for LinkReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.new_ino, state);
            ::core::hash::Hash::hash(&self.name, state)
        }
    }
    impl ::prost::Message for LinkReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.new_ino != 0u64 {
                ::prost::encoding::uint64::encode(2u32, &self.new_ino, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(3u32, &self.name, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "LinkReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.new_ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "new_ino");
                            error
                        })
                }
                3u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.new_ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(2u32, &self.new_ino)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(3u32, &self.name)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.new_ino = 0u64;
            self.name.clear();
        }
    }
    impl ::core::default::Default for LinkReq {
        fn default() -> Self {
            LinkReq {
                ino: 0u64,
                new_ino: 0u64,
                name: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for LinkReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("LinkReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.new_ino)
                };
                builder.field("new_ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct LinkRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for LinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for LinkRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<::core::option::Option<FileAttr>>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for LinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for LinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for LinkRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for LinkRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for LinkRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state)
        }
    }
    impl ::prost::Message for LinkRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "LinkRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
        }
    }
    impl ::core::default::Default for LinkRes {
        fn default() -> Self {
            LinkRes {
                found: false,
                attr: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for LinkRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("LinkRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct SymlinkReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
        #[prost(string, tag = "3")]
        pub target: ::prost::alloc::string::String,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for SymlinkReq {
        #[inline]
        fn clone(&self) -> Self {
            SymlinkReq {
                ino: ::core::clone::Clone::clone(&self.ino),
                name: ::core::clone::Clone::clone(&self.name),
                target: ::core::clone::Clone::clone(&self.target),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for SymlinkReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for SymlinkReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino && self.name == other.name
                && self.target == other.target
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for SymlinkReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::string::String>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for SymlinkReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state);
            ::core::hash::Hash::hash(&self.name, state);
            ::core::hash::Hash::hash(&self.target, state)
        }
    }
    impl ::prost::Message for SymlinkReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
            if self.name != "" {
                ::prost::encoding::string::encode(2u32, &self.name, buf);
            }
            if self.target != "" {
                ::prost::encoding::string::encode(3u32, &self.target, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "SymlinkReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.name;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "name");
                            error
                        })
                }
                3u32 => {
                    let mut value = &mut self.target;
                    ::prost::encoding::string::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "target");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
                + if self.name != "" {
                    ::prost::encoding::string::encoded_len(2u32, &self.name)
                } else {
                    0
                }
                + if self.target != "" {
                    ::prost::encoding::string::encoded_len(3u32, &self.target)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
            self.name.clear();
            self.target.clear();
        }
    }
    impl ::core::default::Default for SymlinkReq {
        fn default() -> Self {
            SymlinkReq {
                ino: 0u64,
                name: ::prost::alloc::string::String::new(),
                target: ::prost::alloc::string::String::new(),
            }
        }
    }
    impl ::core::fmt::Debug for SymlinkReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("SymlinkReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.name)
                };
                builder.field("name", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.target)
                };
                builder.field("target", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct SymlinkRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(message, optional, tag = "2")]
        pub attr: ::core::option::Option<FileAttr>,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for SymlinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for SymlinkRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<::core::option::Option<FileAttr>>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for SymlinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for SymlinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for SymlinkRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.attr == other.attr
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for SymlinkRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::core::option::Option<FileAttr>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for SymlinkRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.attr, state)
        }
    }
    impl ::prost::Message for SymlinkRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if let Some(ref msg) = self.attr {
                ::prost::encoding::message::encode(2u32, msg, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "SymlinkRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.attr;
                    ::prost::encoding::message::merge(
                            wire_type,
                            value.get_or_insert_with(::core::default::Default::default),
                            buf,
                            ctx,
                        )
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "attr");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + self
                    .attr
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(2u32, msg))
        }
        fn clear(&mut self) {
            self.found = false;
            self.attr = ::core::option::Option::None;
        }
    }
    impl ::core::default::Default for SymlinkRes {
        fn default() -> Self {
            SymlinkRes {
                found: false,
                attr: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for SymlinkRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("SymlinkRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = &self.attr;
                builder.field("attr", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct ReadlinkReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for ReadlinkReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReadlinkReq {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for ReadlinkReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReadlinkReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReadlinkReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for ReadlinkReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for ReadlinkReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state)
        }
    }
    impl ::prost::Message for ReadlinkReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReadlinkReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
        }
    }
    impl ::core::default::Default for ReadlinkReq {
        fn default() -> Self {
            ReadlinkReq { ino: 0u64 }
        }
    }
    impl ::core::fmt::Debug for ReadlinkReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReadlinkReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct ReadlinkRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
        #[prost(bytes = "vec", tag = "2")]
        pub data: ::prost::alloc::vec::Vec<u8>,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReadlinkRes {
        #[inline]
        fn clone(&self) -> Self {
            ReadlinkRes {
                found: ::core::clone::Clone::clone(&self.found),
                data: ::core::clone::Clone::clone(&self.data),
            }
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReadlinkRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReadlinkRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found && self.data == other.data
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for ReadlinkRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<::prost::alloc::vec::Vec<u8>>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for ReadlinkRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state);
            ::core::hash::Hash::hash(&self.data, state)
        }
    }
    impl ::prost::Message for ReadlinkRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
            if self.data != b"" as &[u8] {
                ::prost::encoding::bytes::encode(2u32, &self.data, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReadlinkRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                2u32 => {
                    let mut value = &mut self.data;
                    ::prost::encoding::bytes::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "data");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
                + if self.data != b"" as &[u8] {
                    ::prost::encoding::bytes::encoded_len(2u32, &self.data)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.found = false;
            self.data.clear();
        }
    }
    impl ::core::default::Default for ReadlinkRes {
        fn default() -> Self {
            ReadlinkRes {
                found: false,
                data: ::core::default::Default::default(),
            }
        }
    }
    impl ::core::fmt::Debug for ReadlinkRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReadlinkRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.data)
                };
                builder.field("data", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct ReleaseReq {
        #[prost(uint64, tag = "1")]
        pub ino: u64,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for ReleaseReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReleaseReq {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for ReleaseReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReleaseReq {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReleaseReq {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.ino == other.ino
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for ReleaseReq {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for ReleaseReq {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.ino, state)
        }
    }
    impl ::prost::Message for ReleaseReq {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.ino != 0u64 {
                ::prost::encoding::uint64::encode(1u32, &self.ino, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReleaseReq";
            match tag {
                1u32 => {
                    let mut value = &mut self.ino;
                    ::prost::encoding::uint64::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "ino");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.ino != 0u64 {
                    ::prost::encoding::uint64::encoded_len(1u32, &self.ino)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.ino = 0u64;
        }
    }
    impl ::core::default::Default for ReleaseReq {
        fn default() -> Self {
            ReleaseReq { ino: 0u64 }
        }
    }
    impl ::core::fmt::Debug for ReleaseReq {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReleaseReq");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.ino)
                };
                builder.field("ino", &wrapper)
            };
            builder.finish()
        }
    }
    #[allow(clippy::all)]
    pub struct ReleaseRes {
        #[prost(bool, tag = "1")]
        pub found: bool,
    }
    #[automatically_derived]
    #[doc(hidden)]
    #[allow(clippy::all)]
    unsafe impl ::core::clone::TrivialClone for ReleaseRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for ReleaseRes {
        #[inline]
        fn clone(&self) -> Self {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::Copy for ReleaseRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::marker::StructuralPartialEq for ReleaseRes {}
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::PartialEq for ReleaseRes {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.found == other.found
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::cmp::Eq for ReleaseRes {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
        }
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::hash::Hash for ReleaseRes {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            ::core::hash::Hash::hash(&self.found, state)
        }
    }
    impl ::prost::Message for ReleaseRes {
        #[allow(unused_variables)]
        fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
            if self.found != false {
                ::prost::encoding::bool::encode(1u32, &self.found, buf);
            }
        }
        #[allow(unused_variables)]
        fn merge_field(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::wire_type::WireType,
            buf: &mut impl ::prost::bytes::Buf,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            const STRUCT_NAME: &'static str = "ReleaseRes";
            match tag {
                1u32 => {
                    let mut value = &mut self.found;
                    ::prost::encoding::bool::merge(wire_type, value, buf, ctx)
                        .map_err(|mut error| {
                            error.push(STRUCT_NAME, "found");
                            error
                        })
                }
                _ => ::prost::encoding::skip_field(wire_type, tag, buf, ctx),
            }
        }
        #[inline]
        fn encoded_len(&self) -> usize {
            0
                + if self.found != false {
                    ::prost::encoding::bool::encoded_len(1u32, &self.found)
                } else {
                    0
                }
        }
        fn clear(&mut self) {
            self.found = false;
        }
    }
    impl ::core::default::Default for ReleaseRes {
        fn default() -> Self {
            ReleaseRes { found: false }
        }
    }
    impl ::core::fmt::Debug for ReleaseRes {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let mut builder = f.debug_struct("ReleaseRes");
            let builder = {
                let wrapper = {
                    #[allow(non_snake_case)]
                    fn ScalarWrapper<T>(v: T) -> T {
                        v
                    }
                    ScalarWrapper(&self.found)
                };
                builder.field("found", &wrapper)
            };
            builder.finish()
        }
    }
    /// Generated client implementations.
    pub mod fs_lookup_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsLookupClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsLookupClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsLookupClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsLookupClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsLookupClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsLookupClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsLookupClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsLookupClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsLookupClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn lookup(
                &mut self,
                request: impl tonic::IntoRequest<super::LookupReq>,
            ) -> std::result::Result<tonic::Response<super::LookupRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsLookup/Lookup",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsLookup", "Lookup"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_lookup_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsLookupServer.
        pub trait FsLookup: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn lookup<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::LookupReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::LookupRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsLookupServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsLookupServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsLookupServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsLookupServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsLookupServer<T>
        where
            T: FsLookup,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsLookup/Lookup" => {
                        #[allow(non_camel_case_types)]
                        struct LookupSvc<T: FsLookup>(pub Arc<T>);
                        impl<T: FsLookup> tonic::server::UnaryService<super::LookupReq>
                        for LookupSvc<T> {
                            type Response = super::LookupRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::LookupReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsLookup>::lookup(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = LookupSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsLookupServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsLookup";
        impl<T> tonic::server::NamedService for FsLookupServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_getattr_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsGetattrClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsGetattrClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsGetattrClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsGetattrClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsGetattrClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsGetattrClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsGetattrClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsGetattrClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsGetattrClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn getattr(
                &mut self,
                request: impl tonic::IntoRequest<super::GetattrReq>,
            ) -> std::result::Result<tonic::Response<super::GetattrRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsGetattr/Getattr",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsGetattr", "Getattr"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_getattr_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsGetattrServer.
        pub trait FsGetattr: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn getattr<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::GetattrReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::GetattrRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsGetattrServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsGetattrServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsGetattrServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsGetattrServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsGetattrServer<T>
        where
            T: FsGetattr,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsGetattr/Getattr" => {
                        #[allow(non_camel_case_types)]
                        struct GetattrSvc<T: FsGetattr>(pub Arc<T>);
                        impl<T: FsGetattr> tonic::server::UnaryService<super::GetattrReq>
                        for GetattrSvc<T> {
                            type Response = super::GetattrRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::GetattrReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsGetattr>::getattr(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = GetattrSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsGetattrServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsGetattr";
        impl<T> tonic::server::NamedService for FsGetattrServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_setattr_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsSetattrClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsSetattrClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsSetattrClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsSetattrClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsSetattrClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsSetattrClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsSetattrClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsSetattrClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsSetattrClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn setattr(
                &mut self,
                request: impl tonic::IntoRequest<super::SetattrReq>,
            ) -> std::result::Result<tonic::Response<super::SetattrRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsSetattr/Setattr",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsSetattr", "Setattr"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_setattr_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsSetattrServer.
        pub trait FsSetattr: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn setattr<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::SetattrReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::SetattrRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsSetattrServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsSetattrServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsSetattrServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsSetattrServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsSetattrServer<T>
        where
            T: FsSetattr,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsSetattr/Setattr" => {
                        #[allow(non_camel_case_types)]
                        struct SetattrSvc<T: FsSetattr>(pub Arc<T>);
                        impl<T: FsSetattr> tonic::server::UnaryService<super::SetattrReq>
                        for SetattrSvc<T> {
                            type Response = super::SetattrRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::SetattrReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsSetattr>::setattr(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = SetattrSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsSetattrServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsSetattr";
        impl<T> tonic::server::NamedService for FsSetattrServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_create_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsCreateClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsCreateClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsCreateClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsCreateClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsCreateClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsCreateClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsCreateClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsCreateClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsCreateClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn create(
                &mut self,
                request: impl tonic::IntoRequest<super::CreateReq>,
            ) -> std::result::Result<tonic::Response<super::CreateRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsCreate/Create",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsCreate", "Create"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_create_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsCreateServer.
        pub trait FsCreate: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn create<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::CreateReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::CreateRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsCreateServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsCreateServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsCreateServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsCreateServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsCreateServer<T>
        where
            T: FsCreate,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsCreate/Create" => {
                        #[allow(non_camel_case_types)]
                        struct CreateSvc<T: FsCreate>(pub Arc<T>);
                        impl<T: FsCreate> tonic::server::UnaryService<super::CreateReq>
                        for CreateSvc<T> {
                            type Response = super::CreateRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::CreateReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsCreate>::create(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = CreateSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsCreateServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsCreate";
        impl<T> tonic::server::NamedService for FsCreateServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_mkdir_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsMkdirClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsMkdirClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsMkdirClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsMkdirClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsMkdirClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsMkdirClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsMkdirClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsMkdirClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsMkdirClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn mkdir(
                &mut self,
                request: impl tonic::IntoRequest<super::MkdirReq>,
            ) -> std::result::Result<tonic::Response<super::MkdirRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsMkdir/Mkdir",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsMkdir", "Mkdir"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_mkdir_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsMkdirServer.
        pub trait FsMkdir: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn mkdir<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::MkdirReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::MkdirRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsMkdirServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsMkdirServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsMkdirServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsMkdirServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsMkdirServer<T>
        where
            T: FsMkdir,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsMkdir/Mkdir" => {
                        #[allow(non_camel_case_types)]
                        struct MkdirSvc<T: FsMkdir>(pub Arc<T>);
                        impl<T: FsMkdir> tonic::server::UnaryService<super::MkdirReq>
                        for MkdirSvc<T> {
                            type Response = super::MkdirRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::MkdirReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsMkdir>::mkdir(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = MkdirSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsMkdirServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsMkdir";
        impl<T> tonic::server::NamedService for FsMkdirServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_rmdir_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsRmdirClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsRmdirClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsRmdirClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsRmdirClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsRmdirClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsRmdirClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsRmdirClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsRmdirClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsRmdirClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn rmdir(
                &mut self,
                request: impl tonic::IntoRequest<super::RmdirReq>,
            ) -> std::result::Result<tonic::Response<super::RmdirRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsRmdir/Rmdir",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsRmdir", "Rmdir"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_rmdir_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsRmdirServer.
        pub trait FsRmdir: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn rmdir<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::RmdirReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::RmdirRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsRmdirServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsRmdirServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsRmdirServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsRmdirServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsRmdirServer<T>
        where
            T: FsRmdir,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsRmdir/Rmdir" => {
                        #[allow(non_camel_case_types)]
                        struct RmdirSvc<T: FsRmdir>(pub Arc<T>);
                        impl<T: FsRmdir> tonic::server::UnaryService<super::RmdirReq>
                        for RmdirSvc<T> {
                            type Response = super::RmdirRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::RmdirReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsRmdir>::rmdir(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = RmdirSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsRmdirServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsRmdir";
        impl<T> tonic::server::NamedService for FsRmdirServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_unlink_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsUnlinkClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsUnlinkClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsUnlinkClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsUnlinkClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsUnlinkClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsUnlinkClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsUnlinkClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsUnlinkClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsUnlinkClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn unlink(
                &mut self,
                request: impl tonic::IntoRequest<super::UnlinkReq>,
            ) -> std::result::Result<tonic::Response<super::UnlinkRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsUnlink/Unlink",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsUnlink", "Unlink"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_unlink_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsUnlinkServer.
        pub trait FsUnlink: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn unlink<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::UnlinkReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::UnlinkRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsUnlinkServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsUnlinkServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsUnlinkServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsUnlinkServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsUnlinkServer<T>
        where
            T: FsUnlink,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsUnlink/Unlink" => {
                        #[allow(non_camel_case_types)]
                        struct UnlinkSvc<T: FsUnlink>(pub Arc<T>);
                        impl<T: FsUnlink> tonic::server::UnaryService<super::UnlinkReq>
                        for UnlinkSvc<T> {
                            type Response = super::UnlinkRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::UnlinkReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsUnlink>::unlink(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = UnlinkSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsUnlinkServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsUnlink";
        impl<T> tonic::server::NamedService for FsUnlinkServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_rename_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsRenameClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsRenameClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsRenameClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsRenameClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsRenameClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsRenameClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsRenameClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsRenameClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsRenameClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn rename(
                &mut self,
                request: impl tonic::IntoRequest<super::RenameReq>,
            ) -> std::result::Result<tonic::Response<super::RenameRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsRename/Rename",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsRename", "Rename"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_rename_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsRenameServer.
        pub trait FsRename: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn rename<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::RenameReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::RenameRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsRenameServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsRenameServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsRenameServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsRenameServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsRenameServer<T>
        where
            T: FsRename,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsRename/Rename" => {
                        #[allow(non_camel_case_types)]
                        struct RenameSvc<T: FsRename>(pub Arc<T>);
                        impl<T: FsRename> tonic::server::UnaryService<super::RenameReq>
                        for RenameSvc<T> {
                            type Response = super::RenameRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::RenameReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsRename>::rename(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = RenameSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsRenameServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsRename";
        impl<T> tonic::server::NamedService for FsRenameServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_read_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsReadClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReadClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsReadClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsReadClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsReadClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsReadClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsReadClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsReadClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsReadClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn read(
                &mut self,
                request: impl tonic::IntoRequest<super::ReadReq>,
            ) -> std::result::Result<tonic::Response<super::ReadRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsRead/Read",
                );
                let mut req = request.into_request();
                req.extensions_mut().insert(GrpcMethod::new("rusty_net.FsRead", "Read"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_read_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsReadServer.
        pub trait FsRead: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn read<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::ReadReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::ReadRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsReadServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReadServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsReadServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsReadServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsReadServer<T>
        where
            T: FsRead,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsRead/Read" => {
                        #[allow(non_camel_case_types)]
                        struct ReadSvc<T: FsRead>(pub Arc<T>);
                        impl<T: FsRead> tonic::server::UnaryService<super::ReadReq>
                        for ReadSvc<T> {
                            type Response = super::ReadRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::ReadReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsRead>::read(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = ReadSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsReadServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsRead";
        impl<T> tonic::server::NamedService for FsReadServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_readdir_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsReaddirClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReaddirClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsReaddirClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsReaddirClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsReaddirClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsReaddirClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsReaddirClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsReaddirClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsReaddirClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn readdir(
                &mut self,
                request: impl tonic::IntoRequest<super::ReaddirReq>,
            ) -> std::result::Result<tonic::Response<super::ReaddirRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsReaddir/Readdir",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsReaddir", "Readdir"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_readdir_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsReaddirServer.
        pub trait FsReaddir: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn readdir<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::ReaddirReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::ReaddirRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsReaddirServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReaddirServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsReaddirServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsReaddirServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsReaddirServer<T>
        where
            T: FsReaddir,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsReaddir/Readdir" => {
                        #[allow(non_camel_case_types)]
                        struct ReaddirSvc<T: FsReaddir>(pub Arc<T>);
                        impl<T: FsReaddir> tonic::server::UnaryService<super::ReaddirReq>
                        for ReaddirSvc<T> {
                            type Response = super::ReaddirRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::ReaddirReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsReaddir>::readdir(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = ReaddirSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsReaddirServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsReaddir";
        impl<T> tonic::server::NamedService for FsReaddirServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_write_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsWriteClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsWriteClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsWriteClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsWriteClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsWriteClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsWriteClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsWriteClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsWriteClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsWriteClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn write(
                &mut self,
                request: impl tonic::IntoRequest<super::WriteReq>,
            ) -> std::result::Result<tonic::Response<super::WriteRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsWrite/Write",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsWrite", "Write"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_write_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsWriteServer.
        pub trait FsWrite: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn write<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::WriteReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::WriteRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsWriteServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsWriteServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsWriteServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsWriteServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsWriteServer<T>
        where
            T: FsWrite,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsWrite/Write" => {
                        #[allow(non_camel_case_types)]
                        struct WriteSvc<T: FsWrite>(pub Arc<T>);
                        impl<T: FsWrite> tonic::server::UnaryService<super::WriteReq>
                        for WriteSvc<T> {
                            type Response = super::WriteRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::WriteReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsWrite>::write(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = WriteSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsWriteServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsWrite";
        impl<T> tonic::server::NamedService for FsWriteServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_link_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsLinkClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsLinkClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsLinkClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsLinkClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsLinkClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsLinkClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsLinkClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsLinkClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsLinkClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn link(
                &mut self,
                request: impl tonic::IntoRequest<super::LinkReq>,
            ) -> std::result::Result<tonic::Response<super::LinkRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsLink/Link",
                );
                let mut req = request.into_request();
                req.extensions_mut().insert(GrpcMethod::new("rusty_net.FsLink", "Link"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_link_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsLinkServer.
        pub trait FsLink: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn link<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::LinkReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::LinkRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsLinkServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsLinkServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsLinkServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsLinkServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsLinkServer<T>
        where
            T: FsLink,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsLink/Link" => {
                        #[allow(non_camel_case_types)]
                        struct LinkSvc<T: FsLink>(pub Arc<T>);
                        impl<T: FsLink> tonic::server::UnaryService<super::LinkReq>
                        for LinkSvc<T> {
                            type Response = super::LinkRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::LinkReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsLink>::link(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = LinkSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsLinkServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsLink";
        impl<T> tonic::server::NamedService for FsLinkServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_symlink_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsSymlinkClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsSymlinkClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsSymlinkClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsSymlinkClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsSymlinkClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsSymlinkClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsSymlinkClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsSymlinkClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsSymlinkClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn symlink(
                &mut self,
                request: impl tonic::IntoRequest<super::SymlinkReq>,
            ) -> std::result::Result<tonic::Response<super::SymlinkRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsSymlink/Symlink",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsSymlink", "Symlink"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_symlink_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsSymlinkServer.
        pub trait FsSymlink: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn symlink<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::SymlinkReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::SymlinkRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsSymlinkServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsSymlinkServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsSymlinkServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsSymlinkServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsSymlinkServer<T>
        where
            T: FsSymlink,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsSymlink/Symlink" => {
                        #[allow(non_camel_case_types)]
                        struct SymlinkSvc<T: FsSymlink>(pub Arc<T>);
                        impl<T: FsSymlink> tonic::server::UnaryService<super::SymlinkReq>
                        for SymlinkSvc<T> {
                            type Response = super::SymlinkRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::SymlinkReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsSymlink>::symlink(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = SymlinkSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsSymlinkServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsSymlink";
        impl<T> tonic::server::NamedService for FsSymlinkServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_readlink_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        pub struct FsReadlinkClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReadlinkClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsReadlinkClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsReadlinkClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsReadlinkClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsReadlinkClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsReadlinkClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsReadlinkClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsReadlinkClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn readlink(
                &mut self,
                request: impl tonic::IntoRequest<super::ReadlinkReq>,
            ) -> std::result::Result<
                tonic::Response<super::ReadlinkRes>,
                tonic::Status,
            > {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsReadlink/Readlink",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsReadlink", "Readlink"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_readlink_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsReadlinkServer.
        pub trait FsReadlink: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn readlink<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::ReadlinkReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::ReadlinkRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        pub struct FsReadlinkServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReadlinkServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsReadlinkServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsReadlinkServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsReadlinkServer<T>
        where
            T: FsReadlink,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsReadlink/Readlink" => {
                        #[allow(non_camel_case_types)]
                        struct ReadlinkSvc<T: FsReadlink>(pub Arc<T>);
                        impl<
                            T: FsReadlink,
                        > tonic::server::UnaryService<super::ReadlinkReq>
                        for ReadlinkSvc<T> {
                            type Response = super::ReadlinkRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::ReadlinkReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsReadlink>::readlink(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = ReadlinkSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsReadlinkServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsReadlink";
        impl<T> tonic::server::NamedService for FsReadlinkServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
    /// Generated client implementations.
    pub mod fs_release_client {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        use tonic::codegen::http::Uri;
        /// TODO: may need just to confirm a close across systems?
        pub struct FsReleaseClient<T> {
            inner: tonic::client::Grpc<T>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReleaseClient<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FsReleaseClient",
                    "inner",
                    &&self.inner,
                )
            }
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for FsReleaseClient<T> {
            #[inline]
            fn clone(&self) -> Self {
                FsReleaseClient {
                    inner: ::core::clone::Clone::clone(&self.inner),
                }
            }
        }
        impl FsReleaseClient<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
        impl<T> FsReleaseClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
        {
            pub fn new(inner: T) -> Self {
                let inner = tonic::client::Grpc::new(inner);
                Self { inner }
            }
            pub fn with_origin(inner: T, origin: Uri) -> Self {
                let inner = tonic::client::Grpc::with_origin(inner, origin);
                Self { inner }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> FsReleaseClient<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<
                            tonic::body::Body,
                        >>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
            {
                FsReleaseClient::new(InterceptedService::new(inner, interceptor))
            }
            /// Compress requests with the given encoding.
            ///
            /// This requires the server to support it otherwise it might respond with an
            /// error.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.send_compressed(encoding);
                self
            }
            /// Enable decompressing responses.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.inner = self.inner.accept_compressed(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_decoding_message_size(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.inner = self.inner.max_encoding_message_size(limit);
                self
            }
            pub async fn release(
                &mut self,
                request: impl tonic::IntoRequest<super::ReleaseReq>,
            ) -> std::result::Result<tonic::Response<super::ReleaseRes>, tonic::Status> {
                self.inner
                    .ready()
                    .await
                    .map_err(|e| {
                        tonic::Status::unknown(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("Service was not ready: {0}", e.into()),
                                )
                            }),
                        )
                    })?;
                let codec = tonic_prost::ProstCodec::default();
                let path = http::uri::PathAndQuery::from_static(
                    "/rusty_net.FsRelease/Release",
                );
                let mut req = request.into_request();
                req.extensions_mut()
                    .insert(GrpcMethod::new("rusty_net.FsRelease", "Release"));
                self.inner.unary(req, path, codec).await
            }
        }
    }
    /// Generated server implementations.
    pub mod fs_release_server {
        #![allow(
            unused_variables,
            dead_code,
            missing_docs,
            clippy::wildcard_imports,
            clippy::let_unit_value,
        )]
        use tonic::codegen::*;
        /// Generated trait containing gRPC methods that should be implemented for use with FsReleaseServer.
        pub trait FsRelease: std::marker::Send + std::marker::Sync + 'static {
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn release<'life0, 'async_trait>(
                &'life0 self,
                request: tonic::Request<super::ReleaseReq>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = std::result::Result<
                            tonic::Response<super::ReleaseRes>,
                            tonic::Status,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
        }
        /// TODO: may need just to confirm a close across systems?
        pub struct FsReleaseServer<T> {
            inner: Arc<T>,
            accept_compression_encodings: EnabledCompressionEncodings,
            send_compression_encodings: EnabledCompressionEncodings,
            max_decoding_message_size: Option<usize>,
            max_encoding_message_size: Option<usize>,
        }
        #[automatically_derived]
        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for FsReleaseServer<T> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "FsReleaseServer",
                    "inner",
                    &self.inner,
                    "accept_compression_encodings",
                    &self.accept_compression_encodings,
                    "send_compression_encodings",
                    &self.send_compression_encodings,
                    "max_decoding_message_size",
                    &self.max_decoding_message_size,
                    "max_encoding_message_size",
                    &&self.max_encoding_message_size,
                )
            }
        }
        impl<T> FsReleaseServer<T> {
            pub fn new(inner: T) -> Self {
                Self::from_arc(Arc::new(inner))
            }
            pub fn from_arc(inner: Arc<T>) -> Self {
                Self {
                    inner,
                    accept_compression_encodings: Default::default(),
                    send_compression_encodings: Default::default(),
                    max_decoding_message_size: None,
                    max_encoding_message_size: None,
                }
            }
            pub fn with_interceptor<F>(
                inner: T,
                interceptor: F,
            ) -> InterceptedService<Self, F>
            where
                F: tonic::service::Interceptor,
            {
                InterceptedService::new(Self::new(inner), interceptor)
            }
            /// Enable decompressing requests with the given encoding.
            #[must_use]
            pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.accept_compression_encodings.enable(encoding);
                self
            }
            /// Compress responses with the given encoding, if the client supports it.
            #[must_use]
            pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
                self.send_compression_encodings.enable(encoding);
                self
            }
            /// Limits the maximum size of a decoded message.
            ///
            /// Default: `4MB`
            #[must_use]
            pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
                self.max_decoding_message_size = Some(limit);
                self
            }
            /// Limits the maximum size of an encoded message.
            ///
            /// Default: `usize::MAX`
            #[must_use]
            pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
                self.max_encoding_message_size = Some(limit);
                self
            }
        }
        impl<T, B> tonic::codegen::Service<http::Request<B>> for FsReleaseServer<T>
        where
            T: FsRelease,
            B: Body + std::marker::Send + 'static,
            B::Error: Into<StdError> + std::marker::Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;
            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>,
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                match req.uri().path() {
                    "/rusty_net.FsRelease/Release" => {
                        #[allow(non_camel_case_types)]
                        struct ReleaseSvc<T: FsRelease>(pub Arc<T>);
                        impl<T: FsRelease> tonic::server::UnaryService<super::ReleaseReq>
                        for ReleaseSvc<T> {
                            type Response = super::ReleaseRes;
                            type Future = BoxFuture<
                                tonic::Response<Self::Response>,
                                tonic::Status,
                            >;
                            fn call(
                                &mut self,
                                request: tonic::Request<super::ReleaseReq>,
                            ) -> Self::Future {
                                let inner = Arc::clone(&self.0);
                                let fut = async move {
                                    <T as FsRelease>::release(&inner, request).await
                                };
                                Box::pin(fut)
                            }
                        }
                        let accept_compression_encodings = self
                            .accept_compression_encodings;
                        let send_compression_encodings = self.send_compression_encodings;
                        let max_decoding_message_size = self.max_decoding_message_size;
                        let max_encoding_message_size = self.max_encoding_message_size;
                        let inner = self.inner.clone();
                        let fut = async move {
                            let method = ReleaseSvc(inner);
                            let codec = tonic_prost::ProstCodec::default();
                            let mut grpc = tonic::server::Grpc::new(codec)
                                .apply_compression_config(
                                    accept_compression_encodings,
                                    send_compression_encodings,
                                )
                                .apply_max_message_size_config(
                                    max_decoding_message_size,
                                    max_encoding_message_size,
                                );
                            let res = grpc.unary(method, req).await;
                            Ok(res)
                        };
                        Box::pin(fut)
                    }
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(
                                tonic::body::Body::default(),
                            );
                            let headers = response.headers_mut();
                            headers
                                .insert(
                                    tonic::Status::GRPC_STATUS,
                                    (tonic::Code::Unimplemented as i32).into(),
                                );
                            headers
                                .insert(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                );
                            Ok(response)
                        })
                    }
                }
            }
        }
        impl<T> Clone for FsReleaseServer<T> {
            fn clone(&self) -> Self {
                let inner = self.inner.clone();
                Self {
                    inner,
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
        /// Generated gRPC service name
        pub const SERVICE_NAME: &str = "rusty_net.FsRelease";
        impl<T> tonic::server::NamedService for FsReleaseServer<T> {
            const NAME: &'static str = SERVICE_NAME;
        }
    }
}
use rusty_net::*;
use rusty_net::fs_lookup_server::{FsLookup, FsLookupServer};
use rusty_net::fs_lookup_client::FsLookupClient;
struct FsLookupService {}
#[automatically_derived]
impl ::core::fmt::Debug for FsLookupService {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "FsLookupService")
    }
}
#[automatically_derived]
impl ::core::default::Default for FsLookupService {
    #[inline]
    fn default() -> Self {
        FsLookupService {}
    }
}
use rusty_net::fs_getattr_server::{FsGetattr, FsGetattrServer};
use rusty_net::fs_getattr_client::FsGetattrClient;
struct FsGetattrService {}
impl FsGetattr for FsGetattService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn getattr<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<GetattrReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<GetattrRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<GetattrRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<GetattrRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server getattr\n"));
                };
                let inputs = request.get_ref();
                let response = GetattrRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_setattr_server::{FsSetattr, FsSetattrServer};
use rusty_net::fs_setattr_client::FsSetattrClient;
struct FsSetattrService {}
impl FsSetattr for FsSetattrService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn setattr<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<SetattrReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<SetattrRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<SetattrRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<SetattrRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server setattr\n"));
                };
                let inputs = request.get_ref();
                let response = SetattrRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_create_server::{FsCreate, FsCreateServer};
use rusty_net::fs_create_client::FsCreateClient;
struct FsCreateService {}
impl FsCreate for FsCreateService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn create<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<CreateReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<CreateRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<CreateRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<CreateRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server create\n"));
                };
                let inputs = request.get_ref();
                let response = CreateRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_mkdir_server::{FsMkdir, FsMkdirServer};
use rusty_net::fs_mkdir_client::FsMkdirClient;
struct FsMkdirService {}
impl FsMkdir for FsMkdirService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn mkdir<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<MkdirReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<MkdirRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<MkdirRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<MkdirRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server mkdir\n"));
                };
                let inputs = request.get_ref();
                let response = MkdirRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_rmdir_server::{FsRmdir, FsRmdirServer};
use rusty_net::fs_rmdir_client::FsRmdirClient;
struct FsRemdirService {}
impl FsRmdir for FsRmdirService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn rmdir<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<RmdirReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<RmdirRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<RmdirRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<RmdirRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server rmdir\n"));
                };
                let inputs = request.get_ref();
                let response = RmdirRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_unlink_server::{FsUnlink, FsUnlinkServer};
use rusty_net::fs_unlink_client::FsUnlinkClient;
struct FsUnlinkService {}
impl FsUnlink for FsUnlinkService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn unlink<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<UnlinkReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<UnlinkRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<UnlinkRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<UnlinkRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server unlink\n"));
                };
                let inputs = request.get_ref();
                let response = UnlinkRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_rename_server::{FsRename, FsRenameServer};
use rusty_net::fs_rename_client::FsRenameClient;
struct FsRenameService {}
impl FsRename for FsRenameService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn rename<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<RenameReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<RenameRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<RenameRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<RenameRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server rename\n"));
                };
                let inputs = request.get_ref();
                let response = RenameRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_read_server::{FsRead, FsReadServer};
use rusty_net::fs_read_client::FsReadClient;
struct FsReadService {}
impl FsRead for FsReadService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn read<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<ReadReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<ReadRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<ReadRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<ReadRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server read\n"));
                };
                let inputs = request.get_ref();
                let response = ReadRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_readdir_server::{FsReaddir, FsReaddirServer};
use rusty_net::fs_readdir_client::FsReaddirClient;
struct FsReaddirService {}
impl FsReaddir for FsReaddirService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn readdir<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<ReaddirReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<ReaddirRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<ReaddirRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<ReaddirRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server readdir\n"));
                };
                let inputs = request.get_ref();
                let response = ReaddirRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_write_server::{FsWrite, FsWriteServer};
use rusty_net::fs_write_client::FsWriteClient;
struct FsWriteService {}
impl FsWrite for FsWriteService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn write<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<WriteReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<WriteRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<WriteRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<WriteRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server write\n"));
                };
                let inputs = request.get_ref();
                let response = WriteRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_link_server::{FsLink, FsLinkServer};
use rusty_net::fs_link_client::FsLinkClient;
struct FsLinkService {}
impl FsLink for FsLinkService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn link<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<LinkReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<LinkRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<LinkRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<LinkRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server link\n"));
                };
                let inputs = request.get_ref();
                let response = LinkRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_symlink_server::{FsSymlink, FsSymlinkServer};
use rusty_net::fs_symlink_client::FsSymlinkClient;
struct FsSymlinkService {}
impl FsSymlink for FsSymlinkService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn symlink<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<SymlinkReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<SymlinkRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<SymlinkRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<SymlinkRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server symlink\n"));
                };
                let inputs = request.get_ref();
                let response = SymlinkRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_readlink_server::{FsReadlink, FsReadlinkServer};
use rusty_net::fs_readlink_client::FsReadlinkClient;
struct FsReadlinkService {}
impl FsReadlink for FsReadlinkService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn readlink<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<ReadlinkReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<ReadlinkRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<ReadlinkRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<ReadlinkRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server readlink\n"));
                };
                let inputs = request.get_ref();
                let response = ReadlinkRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
use rusty_net::fs_release_server::{FsRelease, FsReleaseServer};
use rusty_net::fs_release_client::FsReleaseClient;
struct FsReleaseService {}
impl FsRelease for FsReleaseService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn release<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<ReleaseReq>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<ReleaseRes>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<ReleaseRes>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let request = request;
            let __ret: Result<tonic::Response<ReleaseRes>, tonic::Status> = {
                {
                    ::std::io::_print(format_args!("in server release\n"));
                };
                let inputs = request.get_ref();
                let response = ReleaseRes {};
                Ok(tonic::Response::new(reponse))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = async {
        let args: Vec<String> = env::args().collect();
        if args.len() == 1 {
            {
                ::std::io::_print(format_args!("starting for testing\n"));
            };
            let mountpoint = Path::new("/home/domc/rusty_cloud/fs_test");
            let mut options = Config::default();
            options.mount_options = ::alloc::boxed::box_assume_init_into_vec_unsafe(
                ::alloc::intrinsics::write_box_via_move(
                    ::alloc::boxed::Box::new_uninit(),
                    [MountOption::AutoUnmount],
                ),
            );
            options.acl = SessionACL::All;
            let mounted_fs = fuser::mount(RustyCloud::new(false), mountpoint, &options);
            if let Err(e) = mounted_fs {
                {
                    ::std::io::_print(
                        format_args!(
                            "error encounted when trying to mount the filesystem\n",
                        ),
                    );
                };
                {
                    ::std::io::_print(format_args!("{0}\n", e));
                };
            }
            Ok(())
        } else if args[2] == "host" {
            let addr = "127.0.0.1".parse()?;
            let lookup = FsLookupService::default();
            Server::builder()
                .add_service(FsLookupServer::new(lookup))
                .serve(addr)
                .await?;
            Ok(())
        } else {
            {
                ::std::io::_print(format_args!("connecting to server\n"));
            };
            Ok(())
        }
    };
    let body = {
        if false {
            let _: &dyn ::core::future::Future<
                Output = Result<(), Box<dyn std::error::Error>>,
            > = &body;
        }
        body
    };
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return,
        clippy::unwrap_in_result
    )]
    {
        use tokio::runtime::Builder;
        return Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
