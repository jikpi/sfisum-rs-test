use crate::constants::DD_COMMENT_CHAR;
use crate::file_rep::directory_snapshot::DirectorySnapshot;
use crate::file_rep::file_metadata::FileMetadata;
use crate::file_rep::file_st::FileSt;
use crate::file_rep::hash::HashValue;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

///Returns a DirectorySnapshot from a digest file. The DirectorySnapshot will be filled
/// with all the information from the digest file.
///
/// The base_path parameter is optional and is used to set the base path of the DirectorySnapshot.
/// If the base_path is not provided, the base path will be set to the directory of the digest file.
///
/// Handles Windows/Unix path separators - Windows paths will be converted to Unix paths and vice versa.
///
/// The directory digest format:
///
/// File format, where C is the comment character:
/// C Directory digest generated at {time}
/// <any other comments>
/// C Hash: <hash type>
/// C Size: 2999880, Last modified: 1733589895
/// *\Desktop\test\text.docx 4534bfadb395bc299157d52eac16c368
/// C Size: 2999880, Last modified: 1733589895
/// *\Desktop\test\text.docx 4534bfadb395bc299157d52eac16c368
/// ...
///
/// <any other comments>
pub fn read_parse_dd<H: HashValue>(
    file_path: &PathBuf,
    base_path: Option<&PathBuf>,
) -> io::Result<DirectorySnapshot<H>> {
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    //Skip all lines until 'C Hash: <hash type>'
    let mut hash_type_line = None;
    while let Some(Ok(line)) = lines.next() {
        if line.starts_with(&format!("{} ", DD_COMMENT_CHAR)) {
            //Get the line, and remove the comment and space
            if let Some(hash_str) = line.strip_prefix(&format!("{} ", DD_COMMENT_CHAR)) {
                if hash_str.starts_with("Hash: ") {
                    hash_type_line = Some(line);
                    break;
                }
            }
        }
    }

    ////Get the hash type line if it exists
    let hash_type_line = hash_type_line.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to find hash type in digest file",
        )
    })?;

    //Parse hash type
    let hash_type = hash_type_line
        .strip_prefix(&format!("{} Hash: ", DD_COMMENT_CHAR))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse hash type"))?;

    if !H::parse_hash_type_string(hash_type) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unsupported hash type",
        ));
    }

    let mut files = Vec::new();

    //Parse (metadata x file) entries
    while let Some(Ok(line)) = lines.next() {
        if line.starts_with(&format!("{} ", DD_COMMENT_CHAR)) {
            //Try to parse metadata
            if let Some(metadata_str) = line.strip_prefix(&format!("{} ", DD_COMMENT_CHAR)) {
                if let Ok(metadata) = FileMetadata::new_from_string(metadata_str) {
                    //Parse file entry on the next line
                    if let Some(Ok(file_line)) = lines.next() {
                        //Must not be a comment or empty
                        if !file_line.starts_with(DD_COMMENT_CHAR) && !file_line.trim().is_empty() {
                            //Split at the last space to get the file path and hash
                            if let Some((path_str, hash_str)) = file_line.rsplit_once(' ') {
                                if let Some(hash) = H::new_from_string(hash_str) {
                                    //Remove the '*'
                                    let path_str = if path_str.starts_with('*') {
                                        let path = &path_str[1..];

                                        //Replace separators if the file was generated on windows/unix fs
                                        #[cfg(windows)]
                                        {
                                            path.replace('/', "\\")
                                        }
                                        #[cfg(unix)]
                                        {
                                            path.replace('\\', "/")
                                        }
                                        //note: canonicalize()?
                                    } else {
                                        path_str.to_string()
                                    };

                                    files.push(FileSt::new(
                                        PathBuf::from(path_str),
                                        Some(hash),
                                        metadata,
                                    ));
                                    continue;
                                }
                            }
                        }
                    }
                    //If any of the above fails, return an error
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid metadata + file entry format in digest file",
                    ));
                }
            }
        }
        //Skip any other lines
    }

    if files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No valid file entries found",
        ));
    }

    //Determine base path
    if let Some(base_path) = base_path {
        //Use base path if provided
        Ok(DirectorySnapshot::new(base_path.clone(), files))
    } else if let Some(base_path) = file_path.parent() {
        //Use the directory of the digest file as the base directory
        Ok(DirectorySnapshot::new(base_path.to_path_buf(), files))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            "Failed to determine base path",
        ))
    }
}
