use crate::constants::FILE_BUFFER_SIZE;
use digest::generic_array::GenericArray;
use digest::Digest;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

pub fn hash_file<D>(path: &PathBuf) -> io::Result<GenericArray<u8, D::OutputSize>>
where
    D: Digest,
{
    let mut hasher = D::new();
    let mut file = File::open(path)?;
    let mut buffer = [0; FILE_BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize())
}
