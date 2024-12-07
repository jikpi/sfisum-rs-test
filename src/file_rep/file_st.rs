use crate::file_rep::file_metadata::FileMetadata;
use crate::file_rep::hash::HashValue;
use std::io;
use std::path::PathBuf;


/// Represents a file, whether it exists or not (Filesystem, Directory digest)
/// Has a full path to the file, metadata, and an optional hash
///
///
pub struct FileSt<H>
where
    H: HashValue,
{
    pub path: PathBuf,
    pub hash: Option<H>,
    pub metadata: FileMetadata,
}

impl<H> FileSt<H>
where
    H: HashValue,
{
    pub fn new(path: PathBuf, hash: Option<H>, metadata: FileMetadata) -> Self {
        FileSt {
            path: path,
            hash,
            metadata,
        }
    }

    //when created from a concrete file on disk
    pub fn new_from_concrete(path: PathBuf) -> io::Result<Self> {
        let metadata = path.metadata()?;
        let metadata = FileMetadata::new(metadata.modified()?, metadata.len());
        Ok(FileSt {
            path: path,
            hash: None,
            metadata,
        })
    }

    pub fn check_exists(&mut self) -> bool {
        if let Ok(exists) = self.path.try_exists() {
            exists
        } else {
            false
        }
    }

    pub fn update_metadata(&mut self) -> io::Result<()> {
        match self.path.metadata() {
            Ok(metadata) => {
                self.metadata = FileMetadata::new(metadata.modified()?, metadata.len());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn calc_hash(&mut self) -> io::Result<()> {
        match H::new_hash_file(&self.path) {
            Ok(hash) => {
                self.hash = Some(hash);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
