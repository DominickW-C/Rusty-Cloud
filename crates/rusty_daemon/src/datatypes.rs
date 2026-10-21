use fuser::{
    FileAttr,
};

use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;

pub enum NodeKind {
    //file data
    File { file_data: Vec<u8> },
    //if it was directory, name and inodeno
    Directory { children: HashMap<String, u64> },
    //symlink
    Symlink { target_ino: String }
}

pub struct FileNode {
    pub attr: FileAttr,
    pub file_type: NodeKind,
    pub path: String,
    pub hash: Option<sha2::digest::Output<Sha256>>
}

pub struct RustyCloud {
    pub f_node: Mutex<HashMap<fuser::INodeNo, FileNode>>,
    pub next_inode: Mutex<u64>,
    pub free_inode: Mutex<Vec<u64>>,
    //TODO: make a free_fh
    pub next_fh: AtomicU64,
}


#[derive(PartialEq)]
pub enum GroupType {
    Same,
    New,
    Deleted
}

pub struct FileGrouped {
    pub data_group: Vec<Vec<u8>>,
    //will be a vector where the index matches the group in data
    pub group_type: Vec<GroupType>,
    //tracks the lines at which a group break happened, going off the new file
    pub group_breaks: Vec<usize>,
    pub made_from: sha2::digest::Output<Sha256>
}
