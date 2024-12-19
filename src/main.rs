use crate::sfisum_instance::Sfisum;

mod constants;
mod engine;
mod file_rep;
mod sfisum_instance;
mod util;

fn main() {
    let instance: Sfisum = Sfisum::new();
    instance.launch_cui();
}
