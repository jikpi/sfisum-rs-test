# Generate

- Create directory snapshot
- Generate it
- Export it

# Check

- Load directory snapshot
- Validate hash
- Print inaccessible files
- Print invalid files

# Fast refresh

- Load directory snapshot
- Mark all files as 'snapshot'
- Walk the actual directory
- Files found in the walk that exist in the directory snapshot are marked as 'exists'
    - However, if their metadata doesn't match, they get marked as 'dirty'
- New files in the walk that don't exist in the directory snapshot are marked as 'new'

Then, the following is done:

- 'exists' files are ignored (their hashes are not computed in Fast refresh - that's what makes it fast)
- 'dirty' files have their hashes computed. If it matches, they get marked as 'valid'. If not, then:
    - If their size or date modified differs, they get marked as 'potentially invalid - s/d/sd'
        - If these metadata are identical, they get marked as 'invalid'
- 'new' files have their hashes computed

Then 'snapshot' file hashes and 'new' file hashes must be cross compared. Before that though, they must be grouped if
there are duplicate files (same hash, different path) within each of the 2 categories. This leaves us with file groups,
though most of these are still singular files.

After a cross comparison, when a match is found, we have the following scenario:

- Group A with hash 'X', known as 'snapshot'
- Group B with hash 'X', known as 'new'
    - Group A is thus re-marked to 'moved' and group B is re-marked as 'found'.

Now, the only problematic files are 'snapshot' -> These files were either deleted or worse - corrupted after being
moved. The corrupted moved file would still be marked as 'new'. It is now no longer necessary to treat the files as
groups from now, and they will be compared on individual level. Now, the following will be done:

- All 'snapshot' files have their size, name (eg 'file.txt') and date compared with 'new' files
- If one or more matches, it is noted
- The matches are then prioritized, with the priority being the amount of attributes that match.

A log is generated with all the information gathered.

After all this is done, in the DD, these are kept:

- All 'exists' files
- All formerly 'dirty' files with hashes created in this session
- All 'found' files
- All 'new' files

Parallel hashing as a method that takes the Vec as mutable and just does stuff as it sees fit.

# Full refresh
Same as fast refresh, but all 'exists' files are instead 'dirty'.

Symlinks?
For snapshot files suspected of corruption, use partial checksums?