use crate::engine::engine::{Engine, EngineAny};
use crate::file_rep::hash::md5::HashMD5;
use crate::file_rep::hash_def::HashType;

pub fn create_engine(hash_type: HashType) -> Box<dyn EngineAny> {
    match hash_type {
        HashType::MD5 => Box::new(Engine::<HashMD5>::new(HashType::MD5)),
    }
}
