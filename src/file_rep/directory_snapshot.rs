use crate::file_rep::file_metadata::FileMetadata;
use crate::file_rep::file_st::FileSt;
use std::path::PathBuf;
use std::{fs, io};
use crate::file_rep::hash_def::HashValue;

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

    //generate a snapshot of the directory
    pub fn generate_from_path(&mut self) -> io::Result<()> {
        //check if directory exists
        if !self.base_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Directory does not exist",
            ));
        }

        //clear files
        self.files.clear();

        let dir = self.base_path.clone();

        match self.generate_directory_rec(&dir) {
            Err(ioerror) => {
                self.files.clear();
                return Err(ioerror);
            }
            _ => {}
        }

        //check if any files were found
        match self.files.len() {
            0 => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No files found in directory",
            )),
            _ => Ok(()),
        }
    }

    fn generate_directory_rec(&mut self, dir: &PathBuf) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let metadata = path.metadata()?;
                    let metadata = FileMetadata::new(metadata.modified()?, metadata.len());

                    let file = FileSt::new(path, None, metadata);
                    self.files.push(file);
                } else if path.is_dir() {
                    self.generate_directory_rec(&path)?;
                }
            }
        }
        Ok(())
    }
}
