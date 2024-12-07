use std::path::{Path, PathBuf};

fn join_paths<P: AsRef<Path>>(base_path: P, relative_path: P) -> PathBuf {
    base_path.as_ref().join(relative_path.as_ref())
}

fn remove_base_path<P: AsRef<Path>>(base_dir: P, full_path: P) -> Option<PathBuf> {
    let base = base_dir.as_ref();
    let full = full_path.as_ref();

    //a way to clean paths
    // let base_clean = base.components();
    // let full_clean = full.components();

    full.strip_prefix(base).ok().map(|p| p.to_path_buf())
}
