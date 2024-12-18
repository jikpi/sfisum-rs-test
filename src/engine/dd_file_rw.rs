use crate::constants::DD_COMMENT_CHAR;
use crate::file_rep::file_metadata::FileMetadata;
use crate::file_rep::file_st::FileSt;
use crate::file_rep::hash_def::{hash_string_to_type, HashType, HashValue};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
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
/// 4534bfadb395bc299157d52eac16c368 *\Desktop\test\text.docx
/// C Size: 2999880, Last modified: 1733589895
/// 4534bfadb395bc299157d52eac16c368 *\Desktop\test\text.docx
/// ...
///
/// <any other comments>
pub fn read_dd<H: HashValue>(
    dd_file_path: &PathBuf,
    base_path: &PathBuf,
) -> io::Result<Vec<FileSt<H>>> {
    let file = File::open(&dd_file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    //Skip all lines until 'C Hash: <hash type>'
    let mut hash_type_line = None;
    let mut current_line = 0;
    while let Some(Ok(line)) = lines.next() {
        current_line += 1;
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
            format!(
                "Failed to find hash type in digest file (processed {} lines)",
                current_line
            ),
        )
    })?;

    //Parse hash type
    let hash_type = hash_type_line
        .strip_prefix(&format!("{} Hash: ", DD_COMMENT_CHAR))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse hash type at line {}", current_line),
            )
        })?;

    if !H::parse_hash_type_string(hash_type) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported hash type at line {}", current_line),
        ));
    }

    let mut files = Vec::new();

    //Parse (metadata x file) entries
    while let Some(Ok(line)) = lines.next() {
        current_line += 1;
        if line.starts_with(&format!("{} ", DD_COMMENT_CHAR)) {
            //Try to parse metadata
            if let Some(metadata_str) = line.strip_prefix(&format!("{} ", DD_COMMENT_CHAR)) {
                if let Ok(metadata) = FileMetadata::new_from_string(metadata_str) {
                    //Parse file entry on the next line
                    if let Some(Ok(file_line)) = lines.next() {
                        current_line += 1;
                        //Must not be a comment or empty
                        if !file_line.starts_with(DD_COMMENT_CHAR) && !file_line.trim().is_empty() {
                            //Split at the first space to get the file path and hash
                            if let Some((hash_str, path_str)) = file_line.split_once(' ') {
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
                                        base_path.join(&path_str),
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
                        format!(
                            "Invalid metadata + file entry format in digest file at line {}",
                            current_line
                        ),
                    ));
                }
            }
        }
        //Skip any other lines
    }

    if files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "No valid file entries found after processing {} lines",
                current_line
            ),
        ));
    }

    Ok(files)
}

pub fn write_dd<H: HashValue>(
    snapshot: &Vec<&FileSt<H>>,
    dd_file_path: &PathBuf,
    base_path: &PathBuf,
) -> io::Result<()> {
    let file = File::create(dd_file_path)?;
    let mut writer = BufWriter::new(file);

    //Title
    writeln!(
        writer,
        "{} Directory digest generated at {} containing {} entries",
        DD_COMMENT_CHAR,
        chrono::Local::now().to_rfc3339(),
        snapshot.len()
    )?;

    //Hash signature
    writeln!(
        writer,
        "{} Hash: {}",
        DD_COMMENT_CHAR,
        H::signature_to_string()
    )?;

    let base_path = base_path.as_path();

    for file in snapshot.iter() {
        //Metadata comment
        writeln!(writer, "{} {}", DD_COMMENT_CHAR, file.metadata.to_string())?;

        //Split path into relative path
        let rel_path = file
            .path
            .strip_prefix(base_path)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let path_str = rel_path.to_string_lossy();

        //Get hash
        let hash_str = file
            .calculated_hash
            .as_ref()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "BUG: File entry missing hash value",
                )
            })?
            .to_string();

        writeln!(writer, "{} *{}", hash_str, path_str)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn parse_dd_hash_type(dd_file_path: &PathBuf) -> Option<HashType> {
    let file = File::open(&dd_file_path).ok()?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    //Skip all lines until 'C Hash: <hash type>'
    while let Some(Ok(line)) = lines.next() {
        if line.starts_with(&format!("{} ", DD_COMMENT_CHAR)) {
            //Get the line, and remove the comment and space
            if let Some(hash_str) = line.strip_prefix(&format!("{} ", DD_COMMENT_CHAR)) {
                if hash_str.starts_with("Hash: ") {
                    // Extract the hash type
                    if let Some(hash_type_str) = hash_str.strip_prefix("Hash: ") {
                        return hash_string_to_type(hash_type_str);
                    }
                }
            }
        }
    }

    None
}
