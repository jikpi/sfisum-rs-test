use crate::file_rep::directory_snapshot::DirectorySnapshot;
use crate::file_rep::hash::{HashMD5, HashValue};
use std::path::PathBuf;

mod constants;
mod engine;
mod file_rep;
mod sfisum_instance;

fn main() {
    println!("Hello, world!");

    let mut ds: DirectorySnapshot<HashMD5> =
        DirectorySnapshot::new_empty(PathBuf::from("C:\\Users\\"));

    ds.generate_from_path().unwrap();

    for mut file in ds.files {
        file.check_exists();
        file.calc_hash().expect("TODO: panic message");

        println!(
            "File: {}, {}, {}",
            file.path.display(),
            file.metadata.to_string(),
            file.hash.unwrap().to_string()
        );
    }
}
