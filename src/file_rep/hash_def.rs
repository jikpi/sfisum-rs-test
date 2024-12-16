use std::cmp::Ordering;
use std::hash::Hasher;
use std::io;
use std::path::PathBuf;
//alternative: simply hash with Vec<u8> but then its slower

pub enum HashType {
    MD5,
}

pub trait HashValue: Sized + Eq + Ord {
    //create from a slice of bytes
    fn new(bytes: &[u8]) -> Option<Self>;
    fn new_hash_file(path: &PathBuf) -> io::Result<Self>;
    fn new_from_string<S: AsRef<str>>(input: S) -> Option<Self>;

    //equality
    fn equals(&self, other: &Self) -> bool;

    //to sort/compare hashes
    fn compare(&self, other: &Self) -> Ordering;

    //bytes
    fn equals_bytes(&self, bytes: &[u8]) -> bool;

    //to string
    fn to_string(&self) -> String;

    fn hash_type() -> HashType;

    fn parse_hash_type_string<S: AsRef<str>>(input: S) -> bool;

    //custom hashing for rust
    fn hash<H: Hasher>(&self, state: &mut H);
}
