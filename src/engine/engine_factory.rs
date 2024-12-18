use crate::engine::engine::{Engine, EngineAny};
use crate::file_rep::hash::md5::HashMD5;
use crate::file_rep::hash_def::{hash_type_suffix_parse, HashType};

pub fn create_engine(hash_type: HashType) -> Box<dyn EngineAny> {
    match hash_type {
        HashType::MD5 => Box::new(Engine::<HashMD5>::new(HashType::MD5)),
    }
}

pub fn dd_filename_to_hash_type<S: AsRef<str>>(filename: S) -> Option<HashType> {
    let filename = filename.as_ref();
    let split: Vec<&str> = filename.split('.').collect();
    let hash_type_str = split.last().unwrap();

    match hash_type_suffix_parse(hash_type_str) {
        Some(hash_type) => Some(hash_type),
        None => None,
    }
}
