use crate::datatypes::{
    FileGrouped,
    GroupType,
};

use sha2::{Sha256};

fn find_file_lines(file: &[u8]) -> Vec<Vec<u8>> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let mut new_line: Vec<u8> = Vec::new();
    for char in file.iter() {
        //was not a new line
        if *char != 10 {
            new_line.push(*char); 
        } else {
            //was a new line, push and empty new line for lines
            new_line.push(*char); 
            lines.push(new_line.clone());
            new_line.clear();
        }
    }

    lines
}

/*
 * might be helpful later, but i found a simpler solution
fn find_current_group_type(file_group: &FileGrouped, current_line: usize) -> GroupType {
    let mut ret_group_type: GroupType;  
    let mut found_flag = false;
    for (ii, breaks) in file_group.group_breaks.iter().enumerate() {
        println!("make sure ii is working right {}", ii);
        if current_line <= *breaks {
            match file_group.group_type[ii] {
                GroupType
            }
            ret_group_type = file_group.group_type[ii];
            found_flag = true;
            break;
        }
    }
    if !found_flag {
        ret_group_type = file_group.group_type[0];
    }

    ret_group_type
}
*/

//current problem:
//when we are creating a new grouping, and encounter a new group in the old grouping,
//we are going to start adding old stuff into a new group, since we are always going to
//comapre to something that we will never see
//NOTE: ONLY OUTPUT A SAME IF BOTH FILES SHARE IT
//NOTE: OR REMOVE ALL NEW CHANGES FROM A OLD FILE, AND GO OFF THAT FOR THE NEW
//WILL REQUIRE ME TO EVENTUALLY GET IT TO FLIP NEW TO OLD
pub fn split_by_changes(old_file_group: &FileGrouped, new_file_data: Vec<u8>, from_hash: sha2::digest::Output<Sha256>) -> FileGrouped {
    let old_file_data = regroup(old_file_group); 
    //println!("old data gotten {:?}", old_file_data);
    //println!("new data gotten {:?}", new_file_data);
    //vectors are array of ascii codes, need to split into lines
    //
    //this should be only the original file since we are removing the new changes to split correctly
    let old_by_line: Vec<Vec<u8>> = find_file_lines(&old_file_data); 
    println!("old by line was {:?}", old_by_line);
    let new_by_line: Vec<Vec<u8>> = find_file_lines(&new_file_data); 
    println!("new by line was {:?}", new_by_line);
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

    //make sure that empty files don't crash the FS
    if old_file_group.group_type.is_empty() {
        return text_group;
    }

    //if the old file starts with new, jump to where it was the same
    //for this to work i need something to change things from new to old
    /*
     * TODO: Commented out becasue there sould be no new anymore 
    if old_file_group.group_type[0] == GroupType::New {
        old_iter = old_file_group.group_breaks[0] - 1;
    }
    */
  
    println!("old iter is now {}", old_iter);
    println!("what are our old data groups? {:?}", old_file_group.data_group);

    while old_iter < old_by_line.len() || new_iter < new_by_line.len() {
        //new_group flag will set the group, and when it is not set, we check the index of current
        //group to see what our current one is

        //we are past the end of the file, meaning that there is NOTHING else we can add from the
        //old file and we can just ignore it
        if old_iter >= old_by_line.len() {
            //loop through the new file until it is done, will be killed by the while loop condition
            //since new and old iter will be too bid

            let group_type_size = text_group.group_type.len();
            //if we were in a same group, we have to clear it, if this isn't true, it was a new
            //group anyways
            //TODO: causing a crash
            if text_group.group_type[group_type_size - 1] == GroupType::Same {
                text_group.group_type.push(GroupType::New);
                text_group.data_group.push(new_group.clone());
                new_group.clear();
            }
            new_group.extend_from_slice(&new_by_line[new_iter]);
            new_iter += 1;
            continue;
        }

        //new file is past end of file, meaning we are looking at deletes now
        //TODO: account for deletes
        if new_iter >= new_by_line.len() {
            old_iter += 1;
            continue;
        }
        println!("old_iter: {}", old_iter);
        println!("new_iter: {}", new_iter);
        println!("old line: {:?}", old_by_line[old_iter]);
        println!("new line: {:?}", new_by_line[new_iter]);
        println!("group_type: {:?}", text_group.group_type);
        println!("new_group: {:?}", new_group);

        //encountered a same line
        if old_by_line[old_iter] == new_by_line[new_iter] {
            println!("LAST GROUP TYPE: {:?}", text_group.group_type.last());
            if first_line_flag {
                new_group.extend_from_slice(&new_by_line[new_iter]);
                text_group.group_type.push(GroupType::Same);
                old_iter += 1;
                new_iter += 1;
                first_line_flag = false;
                continue;
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
                continue;
            } else {
                //it was different push our block and start a new one
                text_group.data_group.push(new_group.clone());
                text_group.group_type.push(GroupType::Same);
                new_group.clear();
                new_group.extend_from_slice(&new_by_line[new_iter]);
                old_iter += 1;
                new_iter += 1;
                //DO NOT NEED +1 since above accounts for it
                text_group.group_breaks.push(new_iter);
                continue;
            }
        }

        //either something was added or deleted
        if old_by_line[old_iter] != new_by_line[new_iter] {
            if first_line_flag {
                new_group.extend_from_slice(&new_by_line[new_iter]);
                text_group.group_type.push(GroupType::New);
                new_iter += 1;
                first_line_flag = false;
                continue;
            }

            //something was added (only thing now)
            let group_type_size = text_group.group_type.len();
            if text_group.group_type[group_type_size - 1] == GroupType::New {
                //already was in a new group
                new_group.extend_from_slice(&new_by_line[new_iter]);
                new_iter += 1;
                continue;
            } else {
                //starting a new group of new
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

    println!("text_grou is {:?}", text_group.data_group);
    text_group
}

pub fn merge_file(old_file_group: &FileGrouped, new_file_group: &FileGrouped) -> Vec<u8> {
    let mut ret_vec: Vec<u8> = Vec::new();

    let mut old_it = 0;
    let mut new_it = 0;

    println!("old file group was {:?}", old_file_group.data_group);
    println!("new file group was {:?}", new_file_group.data_group);
    while old_it < old_file_group.data_group.len() || new_it < new_file_group.data_group.len() {

        //past end of file for old file, just add the new file stuff
        if old_it >= old_file_group.data_group.len() {
            ret_vec.extend_from_slice(&new_file_group.data_group[new_it]);
            new_it += 1;
            continue;
        }

        //past end of file for new file, meaning a delete happened and we can ignore??
        //TODO: fix
        if new_it >= new_file_group.data_group.len() {
            ret_vec.extend_from_slice(&old_file_group.data_group[old_it]);
            old_it += 1;
            continue;
        }

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
                ret_vec.extend_from_slice(&new_file_group.data_group[new_it]);
                old_it += 1;
                new_it += 1;
            } else {
                panic!("merge found a delete and i haven't gotten there yet");
            }
        }
    }

    ret_vec
}

//temp??? fix to the merging issue
pub fn regroup(file_group: &FileGrouped) -> Vec<u8> {
    let mut ret_vec = Vec::new();    
    /*
    for group in file_group.data_group.iter() {
        ret_vec.extend_from_slice(group);
    } 
    */

    for ii in 0..file_group.data_group.len() {
        if file_group.group_type[ii] == GroupType::New {
            continue;
        }
        ret_vec.extend_from_slice(&file_group.data_group[ii]);
    }

    ret_vec
}
