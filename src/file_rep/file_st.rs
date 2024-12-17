use crate::file_rep::file_metadata::FileMetadata;
use crate::file_rep::hash_def::HashValue;
use std::hash::{Hash, Hasher};
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
    pub loaded_hash: Option<H>,
    pub calculated_hash: Option<H>,
    pub metadata: FileMetadata,
}

impl<H> FileSt<H>
where
    H: HashValue,
{
    pub fn new(path: PathBuf, loaded_hash: Option<H>, metadata: FileMetadata) -> Self {
        FileSt {
            path,
            loaded_hash,
            calculated_hash: None,
            metadata,
        }
    }

    //when created from a concrete file on disk
    pub fn new_from_concrete(path: PathBuf) -> io::Result<Self> {
        let metadata = path.metadata()?;
        let metadata = FileMetadata::new(metadata.modified()?, metadata.len());
        Ok(FileSt {
            path,
            loaded_hash: None,
            calculated_hash: None,
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
                self.calculated_hash = Some(hash);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl<H: HashValue> Hash for FileSt<H> {
    fn hash<T: Hasher>(&self, state: &mut T) {
        match &self.loaded_hash {
            Some(hash) => hash.hash(state),
            None => panic!("BUG: Attempted to hash FileSt with no hash value"),
        }
    }
}

impl<H: HashValue> PartialEq for FileSt<H> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.loaded_hash, &other.loaded_hash) {
            (Some(h1), Some(h2)) => h1.equals(h2),
            _ => self.path == other.path,
        }
    }
}

impl<H: HashValue> Eq for FileSt<H> {}
