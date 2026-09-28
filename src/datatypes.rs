use fuser::{
    FileAttr
};

use std::collections::HashMap;
use std::sync::Mutex;

pub enum NodeKind {
    //file data
    File { data: Vec<u8> },
    //if it was directory, name and inodeno
    Directory { children: HashMap<String, u64> }
}

pub struct FileNode {
    pub attr: FileAttr,
    pub file_type: NodeKind,
    pub path: String
}

pub struct RustyCloud {
    pub f_node: Mutex<HashMap<fuser::INodeNo, FileNode>>,
    pub next_inode: Mutex<u64>,
    pub free_inode: Mutex<Vec<u64>>
}
