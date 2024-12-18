use crate::engine::dd_file_rw::{read_dd, write_dd};
use crate::file_rep::hash::md5::HashMD5;
use crate::file_rep::hash_def::HashValue;
use crate::sfisum_instance::Sfisum;
use crate::util::console_text_formatter::{colorize_txt, TextColor};
use std::path::PathBuf;

mod constants;
mod engine;
mod file_rep;
mod sfisum_instance;
mod util;

fn main() {
    let mut instance: Sfisum = Sfisum::new();
    instance.launch_cui();
}
