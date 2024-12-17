use crate::file_rep::directory_snapshot::DirectorySnapshot;
use crate::file_rep::directory_walker::DirectoryWalker;
use crate::file_rep::hash_def::{HashType, HashValue};
use std::io;
use std::io::Stderr;
use std::path::PathBuf;

pub trait EngineAny {
    fn new(hash_type: HashType) -> Self
    where
        Self: Sized;

    fn set_paths(&mut self, dd_file_path: PathBuf, dir_path: PathBuf);
    fn start_generate(&mut self) -> Result<(), String>;
    fn start_validate(&mut self) -> Result<(), String>;
    fn start_fast_refresh(&mut self) -> Result<(), String>;
    fn start_full_refresh(&mut self) -> Result<(), String>;
}

enum Mode {
    Generate,
    Validate,
    FastRefresh,
    FullRefresh,
}

pub struct Engine<H>
where
    H: HashValue,
{
    hash_type: HashType,
    dd_file_path: PathBuf,
    dir_path: PathBuf,
    mode: Option<Mode>,

    primary_ds: DirectorySnapshot<H>,
    secondary_ds: DirectorySnapshot<H>,
}

impl<H: HashValue> EngineAny for Engine<H>
where
    H: HashValue,
{
    fn new(hash_type: HashType) -> Self
    where
        Self: Sized,
    {
        Engine {
            hash_type,
            dd_file_path: PathBuf::new(),
            dir_path: PathBuf::new(),
            primary_ds: DirectorySnapshot::new_empty(PathBuf::new()),
            secondary_ds: DirectorySnapshot::new_empty(PathBuf::new()),
            mode: None,
        }
    }

    fn set_paths(&mut self, dd_file_path: PathBuf, dir_path: PathBuf) {
        self.dd_file_path = dd_file_path;
        self.dir_path = dir_path;
    }

    fn start_generate(&mut self) -> Result<(), String> {
        if let None = self.mode {
            self.mode = Some(Mode::Generate);
        } else {
            return Err("Engine is already in a mode".to_string());
        }

        let mut dir_walker: DirectoryWalker<H> = DirectoryWalker::new(self.dir_path.clone());
        match dir_walker.walk() {
            Err(e) => return Err(format!("Error when walking the directory: {}", e)),
            _ => {}
        }

        self.primary_ds = DirectorySnapshot::new(self.dir_path.clone(), dir_walker.into_files());

        Ok(())
    }

    fn start_validate(&mut self) -> Result<(), String> {
        todo!()
    }

    fn start_fast_refresh(&mut self) -> Result<(), String> {
        todo!()
    }

    fn start_full_refresh(&mut self) -> Result<(), String> {
        todo!()
    }
}
