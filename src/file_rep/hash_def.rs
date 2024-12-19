use std::hash::{Hash};
use std::io;
use std::path::PathBuf;
//alternative: simply hash with Vec<u8> but then its slower

pub enum HashType {
    MD5,
}

pub fn hash_type_suffix_parse<S: AsRef<str>>(input: S) -> Option<HashType> {
    match input.as_ref() {
        "ddmd5" => Some(HashType::MD5),
        _ => None,
    }
}

pub fn hash_type_to_suffix(hash_type: &HashType) -> &'static str {
    match hash_type {
        HashType::MD5 => "ddmd5",
    }
}

pub fn hash_string_to_type<S: AsRef<str>>(input: S) -> Option<HashType> {
    match input.as_ref() {
        "md5" => Some(HashType::MD5),
        _ => None,
    }
}

pub trait HashValue: Sized + Eq + Hash + Clone {
    fn new_hash_file(path: &PathBuf) -> io::Result<Self>;
    fn new_from_string<S: AsRef<str>>(input: S) -> Option<Self>;

    //equality
    fn equals(&self, other: &Self) -> bool;

    //to string
    fn to_string(&self) -> String;

    fn parse_hash_type_string<S: AsRef<str>>(input: S) -> bool;

    fn signature_to_string() -> &'static str;
}
