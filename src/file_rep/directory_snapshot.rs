use crate::file_rep::file_st::FileSt;
use crate::file_rep::hash::HashValue;
use std::path::PathBuf;
use std::{fs, io};

pub struct DirectorySnapshot<H>
where
    H: HashValue,
{
    pub dir: PathBuf,
    pub files: Vec<FileSt<H>>,
}

impl<H> DirectorySnapshot<H>
where
    H: HashValue,
{
    pub fn new(path: PathBuf) -> Self {
        DirectorySnapshot {
            dir: path,
            files: Vec::new(),
        }
    }

    //generate a snapshot of the directory
    pub fn generate(&mut self) -> io::Result<()> {
        //check if directory exists
        if !self.dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Directory does not exist",
            ));
        }

        //clear files
        self.files.clear();

        let dir = self.dir.clone();

        self.generate_directory_rec(&dir)?;


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
                    let file = FileSt::new_from_concrete(path)?;
                    self.files.push(file);
                } else if path.is_dir() {
                    self.generate_directory_rec(&path)?;
                }
            }
        }
        Ok(())
    }
}
