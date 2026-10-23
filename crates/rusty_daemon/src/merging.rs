use crate::datatypes::{
    FileGrouped,
    GroupType,
};

use sha2::{Sha256};

//called to see if a merge was needed
pub fn merge_needed(old_hash: sha2::digest::Output::<Sha256>, new_hash: sha2::digest::Output::<Sha256>) -> bool {
    println!("old_hash was: {:?}", old_hash);
    println!("old_hash was: {:?}", new_hash);

    if old_hash == new_hash {
        println!("same file, do nothing");
        return false
    }
    println!("differenjt file, merge needed");
    true
}

fn find_file_lines(file: &[u8]) -> Vec<Vec<u8>> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    for char in file.iter() {
        let mut new_line: Vec<u8> = Vec::new();
        //was not a new line
        if *char != 10 {
            new_line.push(*char); 
        } else {
            //was a new line, push and empty new line for lines
            lines.push(new_line.clone());
            new_line.clear();
        }
    }

    lines
}

pub fn split_by_changes(old_file_data: Vec<u8>, new_file_data: Vec<u8>, from_hash: sha2::digest::Output<Sha256>) -> FileGrouped {
    //vectors are array of ascii codes, need to split into lines
    let old_by_line: Vec<Vec<u8>> = find_file_lines(&old_file_data); 
    let new_by_line: Vec<Vec<u8>> = find_file_lines(&new_file_data); 
    let mut old_iter = 0;
    let mut new_iter = 0;

    //TODO: ACCOUNT FOR DELETES
    let mut text_group = FileGrouped {
        data_group: Vec::new(),
        group_type: Vec::new(),
        group_breaks: Vec::new(),
        made_from: from_hash
    };

    let mut first_line_flag: bool = true;
    let mut new_group: Vec<u8> = Vec::new();
    while old_iter < old_by_line.len() && new_iter < new_by_line.len() {
        //new_group flag will set the group, and when it is not set, we check the index of current
        //group to see what our current one is

        //encountered a same line
        if old_by_line[old_iter] == new_by_line[new_iter] {
            if first_line_flag {
                new_group.extend_from_slice(&new_by_line[new_iter]);
                text_group.group_type.push(GroupType::Same);
                old_iter += 1;
                new_iter += 1;
                first_line_flag = false;
            }
            //currently at a new group of old stuff, start filling a group, and push the type of
            //group to the text_group
            //check if the last group was same 
            let group_type_size = text_group.group_type.len();
            if text_group.group_type[group_type_size - 1] == GroupType::Same {
                //it was the same, push to new block and continue
                new_group.extend_from_slice(&new_by_line[new_iter]);
                old_iter += 1;
                new_iter += 1;
            } else {
                //it was different push our block and start a new one
                text_group.data_group.push(new_group.clone());
                text_group.group_type.push(GroupType::Same);
                new_group.clear();
                new_group.extend_from_slice(&new_by_line[new_iter]);
                old_iter += 1;
                new_iter += 1;
                //+1 accounts for indexing
                text_group.group_breaks.push(new_iter + 1);
            }
        }

        //either something was added or deleted
        if old_by_line[old_iter] != new_by_line[new_iter] {
            if first_line_flag {
                new_group.extend_from_slice(&new_by_line[new_iter]);
                text_group.group_type.push(GroupType::New);
                new_iter += 1;
                first_line_flag = false;
            }
            //something was added (only thing now)
            let group_type_size = text_group.group_type.len();
            if text_group.group_type[group_type_size - 1] == GroupType::New {
                //already was in a new group
                new_group.extend_from_slice(&new_by_line[new_iter]);
                new_iter += 1;
            } else {
                //starting a new group of new
                text_group.data_group.push(new_group.clone());
                text_group.group_type.push(GroupType::New);
                new_group.clear();
                new_group.extend_from_slice(&new_by_line[new_iter]);
                new_iter += 1;
                text_group.group_breaks.push(new_iter + 1);
            }

        }
    }

    //i don't think it will actually return this but the error is annoying
    text_group
}

pub fn merge_file(old_file_group: &FileGrouped, new_file_group: &FileGrouped) -> Vec<u8> {
    let mut ret_vec: Vec<u8> = Vec::new();

    let mut old_it = 0;
    let mut new_it = 0;

    while old_it < old_file_group.data_group.len() && new_it < new_file_group.data_group.len() {
        //check for new first, prioritize the new files new block first
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
                ret_vec.extend_from_slice(&old_file_group.data_group[old_it]);
                old_it += 1;
                new_it += 1;
            } else {
                panic!("merge found a delete and i haven't gotten there yet");
            }
        }
    }

    ret_vec
}
