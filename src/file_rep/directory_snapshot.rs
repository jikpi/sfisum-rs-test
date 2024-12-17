use crate::file_rep::file_metadata::FileMetadata;
use crate::file_rep::file_st::FileSt;
use crate::file_rep::hash_def::HashValue;
use std::path::PathBuf;
use std::{fs, io};

/// Represents a snapshot of a directory, containing all files and their metadata
/// The directory is created either from a filesystem or a directory digest
/// The path is the base path of the directory
pub struct DirectorySnapshot<H>
where
    H: HashValue,
{
    pub base_path: PathBuf,
    pub files: Vec<FileSt<H>>,
}

impl<H> DirectorySnapshot<H>
where
    H: HashValue,
{
    pub fn new(path: PathBuf, files: Vec<FileSt<H>>) -> Self {
        DirectorySnapshot {
            base_path: path,
            files,
        }
    }

    pub fn new_empty(path: PathBuf) -> Self {
        DirectorySnapshot {
            base_path: path,
            files: Vec::new(),
        }
    }
}
