# Plan for how the project will work

## Requirements

- files can be stored in the filesystem
- When two files are being updated together, both changes get saved together
  - this is done with hashes to compare the state of the file
- filesystem can be connected to as a network drive


## look into mutexs
wiregaurd

## things to do

- download rust FUSE
- get it working
- implement a basic filesystem
  - write, save, read file
- design the structure for the file
  - filename
  - permissions
  - file path
  - current hash
  - the amount of times the file is currently open
  - *will not need the size, size found by finding size of this struct*

*METADATA CAN NOT BE IN A HASH*
this will cause issues
if someone opens a file and closes it but the last time had more users on it
metadata will be different and the filesystem will think something was changed 

## rollback prevention 

- what if someone updates the file and then someone saves an old version?
- rollback protection
- a csv/struct is kept with file names and hashes
- ex
  test.txt,lkj34lkjfh 
  test.txt,wieljb30 
  t.txt,wieljb30 
- these store all the previous hashes of a file
- when we do a hash check and have mismatched hashes, we search this file
- we look for all with the same name, and then compares the hashes
- if none of the hashes match, it is a new change and we update
- if there is a matching hash, it was an old file getting saved, and we ignore the change
- this file will end up getting big, we prevent this as when a file hits no opens, we remove any 
  entries from this file 

## what do you do if you change a file name?


## use cases

### someone opens and closes a file
- the file it opened
- the current open metadata is incremented
- the file is closed, triggers a hash check
- no hash change, decrement amount of users in file and do nothing else

### one person accesses a file at a time, edits it and saves it
- the file it opened
- the current open metadata is incremented
- the file is edited and saved
- the the file data (not including the metadata is hashed)
- the hash is compared, a file change will be detected
- we will check the current amount of users, the only one will be the person who just saved it
- since there is only one version of a file open, we can save their changes without issues
- updates file metadata to reflect the new hash
- decrement the amount of people in the file

### two people open a file, first person makes changes, the second does nothing 
- second person closes the file and triggers a hash check 
- see rollback prevention
- finds out it is a old hash, discard the second save
- decrement the amount of users
- 0 users, wipe entries for this file from rollback prevention table

### two people open a file, first person does nothing and closes, the second makes changes after the first closes 
- this should work like one person editing a file
- the first person saves it and it does nothing besides decrement amount of active users because compare hash noticed same file
- second person saves and hash check notices change, nothing in rollback prevention file, and they are the only user 
- save there change update hash, decrement user

### two people open a file, first person does nothing and keeps it open, the second makes changes while first file open still

### two people open a file, first and second make changes


#### new plan

current file: test.txt with hash a1

user1 opens text.txt with hash a1

user2 opens test.txt with hash a1

user1 saves, generates hash b2
- run through the split function
- saves in a datatype
- check old hash to new hash
  - b2 made from a1, good to save to the filesystem

user2 saves, generate hash c3
- run through the split function, with a1
- saves in a datatype,
- check old hash to new hash
  - c3 was made from a1, current is b2, we gotta do something
  - call merge, takes b2 and c3, and merges them into one file
  - that file is saved to the filesystem, new hash is generated from this

new datatypes for splits will fill up fast, we need a way to remove old ones
- every new save will generate a new hash 
- even if a new save is just a old file, it should trigger a new hash the same as old
  - only on a write call though

When a file is opened from the filesystem, that hashes split does not need to exist anymore
- it was successfully saves, we can look for its file in the split hashmap (probably a hashmap)
  and delete it
