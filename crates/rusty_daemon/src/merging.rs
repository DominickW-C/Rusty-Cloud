use crate::datatypes::{
    FileGrouped,
    GroupType
};

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

pub fn split_by_changes(old_file_data: Vec<u8>, new_file_data: Vec<u8>) -> FileGrouped {
    //vectors are array of ascii codes, need to split into lines
    let old_by_line: Vec<Vec<u8>> = find_file_lines(&old_file_data); 
    let new_by_line: Vec<Vec<u8>> = find_file_lines(&new_file_data); 
    let mut old_iter = 0;
    let mut new_iter = 0;
    //data group we are currently on
    let mut current_group = 0;

    //remove first iter since done here
    //TODO: ACCOUNT DELETES
    let mut text_group = FileGrouped {
        data_group: Vec::new(),
        group_type: Vec::new()
    };

    let mut new_group: bool = true;
    let new_group: Vec<u8> = Vec::new();
    while old_iter < old_by_line.len() && new_iter < new_by_line.len() {
        //new_group flag will set the group, and when it is not set, we check the index of current
        //group to see what our current one is

        if old_by_line[old_iter] == new_by_line[new_iter] {
        }
    }

    //i don't think it will actually return this but the error is annoying
    text_group
}
