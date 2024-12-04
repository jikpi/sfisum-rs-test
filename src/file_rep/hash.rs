use crate::file_rep::file_hasher::hash_file;
use std::cmp::Ordering;
use std::io;
use std::path::PathBuf;
//alternative: simply hash with Vec<u8> but then its slower

pub trait HashValue: Sized {
    //create from a slice of bytes
    fn new(bytes: &[u8]) -> Option<Self>;
    fn new_from_file(path: &PathBuf) -> io::Result<Self>;

    //equality
    fn equals(&self, other: &Self) -> bool;

    //to sort/compare hashes
    fn compare(&self, other: &Self) -> Ordering;

    //bytes
    fn equals_bytes(&self, bytes: &[u8]) -> bool;

    //to string
    fn to_string(&self) -> String;
}

// MD5 ###############################################################
pub struct HashMD5([u8; 16]);

impl HashValue for HashMD5 {
    fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 16 {
            return None;
        }

        let array: [u8; 16] = bytes.try_into().ok()?;

        Some(HashMD5(array))
    }

    fn new_from_file(path: &PathBuf) -> io::Result<Self> {
        let result = hash_file::<md5::Md5>(path)?;
        Ok(Self(result.into())) //directly convert since compile time known size
    }

    fn equals(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn compare(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }

    fn equals_bytes(&self, bytes: &[u8]) -> bool {
        if bytes.len() != 16 {
            return false;
        }
        if let Ok(array) = bytes.try_into() as Result<[u8; 16], _> {
            self.0 == array
        } else {
            false
        }
    }

    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>()
    }
}

impl PartialEq for HashMD5 {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for HashMD5 {}

impl PartialOrd for HashMD5 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for HashMD5 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

// #########################################################################
