use crate::file_rep::file_metadata::FileMetadata;
use crate::file_rep::file_st::FileSt;
use crate::file_rep::hash_def::HashValue;
use std::path::PathBuf;
use std::{fs, io};

pub struct DirectoryWalker<H>
where
    H: HashValue,
{
    pub base_path: PathBuf,
    pub files: Vec<FileSt<H>>,
}

impl<H> DirectoryWalker<H>
where
    H: HashValue,
{
    pub fn new(path: PathBuf) -> Self {
        DirectoryWalker {
            base_path: path,
            files: Vec::new(),
        }
    }

    //walk the directory and collect all files
    pub fn walk(&mut self) -> io::Result<()> {
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

        match self.walk_rec(&dir) {
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

    fn walk_rec(&mut self, dir: &PathBuf) -> io::Result<()> {
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
                    self.walk_rec(&path)?;
                }
            }
        }
        Ok(())
    }

    pub fn into_files(self) -> Vec<FileSt<H>> {
        self.files
    }
}
