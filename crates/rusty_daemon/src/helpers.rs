use fuser::{
    INodeNo,
    FileAttr,
    FileType,
    FileHandle
};

use crate::HashMap;
use crate::NodeKind;
use crate::FileNode;
use crate::datatypes::RustyCloud;

pub fn unwrap_parent_dir(node_map: &mut HashMap<fuser::INodeNo, FileNode>, parent_ino: INodeNo) -> &mut HashMap<String, u64> {
    //might cause issues being always get_mut??
    match node_map.get_mut(&parent_ino) {
        Some(parent_node) => {
            match &mut parent_node.file_type {
                NodeKind::Directory { children } => children,
                NodeKind::File { .. } => panic!("Parent node was a file"),
                NodeKind::Symlink { .. } => panic!("Parent node was a symlink")

            }
        }
        None => panic!("Tried unwrapping parent, failed")
    }
}

pub fn new_inode_no(fs: &RustyCloud) -> u64 {
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

pub fn new_file_node(new_attr: &FileAttr, parent_path: &str, node_name: &str) -> Option<FileNode> {
    let mut file_path = parent_path.to_owned() + node_name;    
    let new_f_node: FileNode;
    let open: Vec<FileHandle> = Vec::new();


    match new_attr.kind {
        FileType::Directory => {
            file_path = file_path.clone() + "/";
            new_f_node = FileNode {
                attr: *new_attr,
                file_type: NodeKind::Directory { children: HashMap::new() },
                path: file_path,
                //WARNING: could be wrong
                current_opens: open
            };
        } 
        FileType::RegularFile => {
            new_f_node = FileNode {
                attr: *new_attr,
                file_type: NodeKind::File { file_data: Vec::new() },
                path: file_path, 
                current_opens: open
            };
        } 
        FileType::Symlink => {
            panic!("symlinks are dealt with in function");
        }
        FileType::NamedPipe | FileType::CharDevice | FileType::BlockDevice | FileType::Socket => {
            println!("ERROR, BAD THING HAPPENED");
            println!("NOT IMPLEMENTED FOR NAMED PIPES, CHAR DEVICES, etc");
            println!("SEE new_file_node");
            return None
        }

    }
    Some(new_f_node)
}

/*
 * works great but no use yet
pub fn find_inode_from_path(node_map: &mut HashMap<fuser::INodeNo, FileNode>, target_path: &str) -> u64{
    let split_path: Vec<&str> = target_path.split('/').collect(); 
    let mut last_parent_ino: u64 = 1;
    for cur_node in split_path.iter() {
        //first is always empty since we start at root
        if cur_node.is_empty() {
            continue;
        }
        //we will always be in a valid parent
        let p_node = node_map.get(&INodeNo(last_parent_ino)).unwrap();

        if let NodeKind::Directory { children } = &p_node.file_type {
            last_parent_ino = *children.get(*cur_node).unwrap();
        }
    }

    last_parent_ino
}
*/

