use crate::constants::{LARGE_FILE_THREADS, SMALL_FILE_SIZE_THRESHOLD, SMALL_FILE_THREADS};
use crate::engine::dd_file_rw::{read_dd, write_dd};
use crate::engine::hash_calc_planner::calculate_hashes;
use crate::file_rep::directory_walker::DirectoryWalker;
use crate::file_rep::file_st::FileSt;
use crate::file_rep::hash_def::{hash_type_to_suffix, HashType, HashValue};
use crate::util::console_text_formatter::{colorize_txt, TextColor};
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub trait EngineAny {
    fn new(hash_type: HashType) -> Self
    where
        Self: Sized;

    fn set_paths(&mut self, dd_file_path: PathBuf, dir_path: PathBuf);
    fn save_dd_file(&self, new_dd_file_dir: Option<PathBuf>) -> Result<(), String>;
    fn start_generate(&mut self) -> Result<(), String>;
    fn start_validate(&mut self) -> Result<(), String>;
    fn start_fast_refresh(&mut self) -> Result<(), String>;
    fn start_full_refresh(&mut self) -> Result<(), String>;

    fn print_log_generate(&self);
    fn print_log_validate(&self);
    fn print_log_fast_refresh(&self);
    fn print_log_full_refresh(&self);

    fn event_count_generate(&self) -> usize;
    fn event_count_validate(&self) -> usize;
    fn event_count_fast_refresh(&self) -> usize;
    fn event_count_full_refresh(&self) -> usize;
}

#[derive(PartialEq)]
enum Mode {
    Generate,
    Validate,
    FastRefresh,
    FullRefresh,
}

pub struct Engine<H>
where
    H: HashValue,
{
    hash_type: HashType,
    dd_file_path: PathBuf,
    base_path: PathBuf,
    mode: Option<Mode>,

    primary_ds: Vec<FileSt<H>>,
    secondary_ds: Vec<FileSt<H>>,

    hashing_error_index: Vec<usize>,
    invalid_hash_index: Vec<usize>,

    dirty_files_index: Vec<usize>,
    dirty_valid_files_index: Vec<usize>,
    dirty_potentially_invalid_s_files_index: Vec<usize>,
    dirty_potentially_invalid_d_files_index: Vec<usize>,
    dirty_potentially_invalid_sd_files_index: Vec<usize>,

    crosscheck_primary_to_secondary_found_index: HashMap<H, (Vec<usize>, Vec<usize>)>,
    crosscheck_secondary_orphans_index: Vec<usize>,
    crosscheck_primary_orphans_index: Vec<usize>,
    crosscheck_secondary_orphan_but_duplicate_index: Vec<usize>,
}

impl<H: HashValue + Sync + Send + std::fmt::Debug> EngineAny for Engine<H>
where
    H: HashValue,
{
    fn new(hash_type: HashType) -> Self
    where
        Self: Sized,
    {
        Engine {
            hash_type,
            dd_file_path: PathBuf::new(),
            base_path: PathBuf::new(),
            primary_ds: Vec::new(),
            secondary_ds: Vec::new(),
            mode: None,
            hashing_error_index: Vec::new(),
            invalid_hash_index: Vec::new(),
            dirty_files_index: Vec::new(),
            dirty_potentially_invalid_s_files_index: Vec::new(),
            dirty_potentially_invalid_d_files_index: Vec::new(),
            dirty_potentially_invalid_sd_files_index: Vec::new(),
            dirty_valid_files_index: Vec::new(),
            crosscheck_primary_to_secondary_found_index: HashMap::new(),
            crosscheck_secondary_orphans_index: Vec::new(),
            crosscheck_primary_orphans_index: Vec::new(),
            crosscheck_secondary_orphan_but_duplicate_index: Vec::new(),
        }
    }

    fn set_paths(&mut self, dd_file_path: PathBuf, dir_path: PathBuf) {
        self.dd_file_path = dd_file_path;
        self.base_path = dir_path;
    }

    fn save_dd_file(&self, new_dd_file_dir: Option<PathBuf>) -> Result<(), String> {
        //Create the dd file path
        let dd_file_path = match new_dd_file_dir {
            //Save in the specified directory
            Some(mut dir) => {
                let now = chrono::Local::now();
                let date_time = now.format("%Y-%m-%d_%H-%M");
                dir.push(format!(
                    "{}.{}",
                    date_time,
                    hash_type_to_suffix(&self.hash_type)
                ));
                dir
            }
            None => {
                //Fallback to the executable directory
                let mut path = std::env::current_exe().unwrap();
                path.pop();
                path.push(format!("digest.{}", hash_type_to_suffix(&self.hash_type)));

                path
            }
        };

        let valid_files: Vec<&FileSt<H>> = self
            .primary_ds
            .iter()
            .enumerate()
            .filter(|(index, _)| !self.hashing_error_index.contains(index))
            .map(|(_, file)| file)
            .collect();

        write_dd(&valid_files, &dd_file_path, &self.base_path)
            .map_err(|e| format!("Failed to write dd file: {}", e))
    }

    // ############################################################################################

    fn start_generate(&mut self) -> Result<(), String> {
        if let None = self.mode {
            self.mode = Some(Mode::Generate);
        } else {
            return Err("BUG: Engine is already in a mode".to_string());
        }

        let mut dir_walker: DirectoryWalker<H> = DirectoryWalker::new(self.base_path.clone());
        match dir_walker.walk() {
            Err(e) => return Err(format!("Error when walking the directory: {}", e)),
            _ => {}
        }

        self.primary_ds = dir_walker.into_files();

        match calculate_hashes(
            &mut self.primary_ds,
            SMALL_FILE_THREADS,
            LARGE_FILE_THREADS,
            SMALL_FILE_SIZE_THRESHOLD,
            None,
        ) {
            Ok(error_indexes) => {
                if let Some(indexes) = error_indexes {
                    self.hashing_error_index = indexes;
                }
            }
            Err(e) => return Err(format!("Failed to calculate hashes: {}", e)),
        }

        Ok(())
    }

    fn start_validate(&mut self) -> Result<(), String> {
        if let None = self.mode {
            self.mode = Some(Mode::Validate);
        } else {
            return Err("BUG: Engine is already in a mode".to_string());
        }

        self.primary_ds = read_dd(&self.dd_file_path, &self.base_path)
            .map_err(|e| format!("Failed to read dd file: {}", e))?;

        match calculate_hashes(
            &mut self.primary_ds,
            SMALL_FILE_THREADS,
            LARGE_FILE_THREADS,
            SMALL_FILE_SIZE_THRESHOLD,
            None,
        ) {
            Ok(error_indexes) => {
                if let Some(indexes) = error_indexes {
                    self.hashing_error_index = indexes;
                }
            }
            Err(e) => return Err(format!("Failed to calculate hashes: {}", e)),
        }

        for (index, file) in self.primary_ds.iter().enumerate() {
            match file.calculated_hash {
                Some(_) => {}
                None => {
                    continue;
                }
            }

            let calculated_hash = file
                .calculated_hash
                .as_ref()
                .expect("BUG: In validation, 'calculated hash' has no hash");
            let loaded_hash = file
                .loaded_hash
                .as_ref()
                .expect("BUG: In validation, 'loaded hash' has no hash");

            if !loaded_hash.equals(calculated_hash) {
                self.invalid_hash_index.push(index);
            }
        }

        Ok(())
    }

    fn start_fast_refresh(&mut self) -> Result<(), String> {
        if let None = self.mode {
            self.mode = Some(Mode::FastRefresh);
        } else {
            return Err("BUG: Engine is already in a mode".to_string());
        }

        let mut dir_walker: DirectoryWalker<H> = DirectoryWalker::new(self.base_path.clone());
        match dir_walker.walk() {
            Err(e) => return Err(format!("Error when walking the directory: {}", e)),
            _ => {}
        }

        //Primary snapshot is from the directory
        self.primary_ds = dir_walker.into_files();

        //Secondary snapshot is from the file
        self.secondary_ds = read_dd(&self.dd_file_path, &self.base_path)
            .map_err(|e| format!("Failed to read dd file: {}", e))?;

        //A hashmap of Path->Index for Primary snap
        let mut primary_paths_index: HashMap<&Path, usize> = HashMap::new();

        for (index, file) in self.primary_ds.iter().enumerate() {
            primary_paths_index.insert(file.path.as_path(), index);
        }

        //A hashmap of Path->Index for Secondary snap
        let mut secondary_paths_index: HashMap<&Path, usize> = HashMap::new();

        for (index, file) in self.secondary_ds.iter().enumerate() {
            secondary_paths_index.insert(file.path.as_path(), index);
        }

        let mut only_in_primary_index: Vec<usize> = Vec::new();
        let mut only_in_secondary_index: Vec<usize> = Vec::new();
        let mut in_both_index: Vec<(usize, usize)> = Vec::new(); // (primary_id, secondary_id)

        //Get the indexes for the files
        for (&path, &primary_idx) in &primary_paths_index {
            match secondary_paths_index.get(path) {
                Some(&secondary_idx) => in_both_index.push((primary_idx, secondary_idx)),
                None => only_in_primary_index.push(primary_idx),
            }
        }

        only_in_secondary_index = secondary_paths_index
            .iter()
            .filter(|(&path, _)| !primary_paths_index.contains_key(path))
            .map(|(_, &idx)| idx)
            .collect();

        let mut dirty_files_index: Vec<(usize, usize)> = Vec::new();

        //Get dirty files (files that have different metadata)
        for (primary_index, secondary_index) in in_both_index.iter() {
            let primary_file = &self.primary_ds[*primary_index];
            let secondary_file = &self.secondary_ds[*secondary_index];

            let primary_secs = primary_file
                .metadata
                .last_modified
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let secondary_secs = secondary_file
                .metadata
                .last_modified
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if primary_file.metadata.size != secondary_file.metadata.size
                || primary_secs != secondary_secs
            {
                dirty_files_index.push((*primary_index, *secondary_index));
                continue;
            }
        }

        let mut files_to_hash_index: Vec<usize> = Vec::new();

        //Add dirty and 'only in primary' files to the list of files to hash
        files_to_hash_index.extend(only_in_primary_index.iter());
        files_to_hash_index.extend(dirty_files_index.iter().map(|(primary_idx, _)| primary_idx));

        //Hash the files
        if !files_to_hash_index.is_empty() {
            match calculate_hashes(
                &mut self.primary_ds,
                SMALL_FILE_THREADS,
                LARGE_FILE_THREADS,
                SMALL_FILE_SIZE_THRESHOLD,
                Some(files_to_hash_index),
            ) {
                Ok(error_indexes) => {
                    if let Some(indexes) = error_indexes {
                        self.hashing_error_index = indexes;
                    }
                }
                Err(e) => return Err(format!("Failed to calculate hashes: {}", e)),
            }
        }

        //Mark files that have invalid hashes to generate a report later

        for (primary_file_index, secondary) in &dirty_files_index {
            //if hash is not the same, and date and size are the same, then the file is invalid
            let primary_file = &self.primary_ds[*primary_file_index];
            let secondary_file = &self.secondary_ds[*secondary];

            let primary_hash = primary_file
                .calculated_hash
                .as_ref()
                .expect("BUG: In fast refresh, 'calculated hash' has no hash");

            let secondary_hash = secondary_file
                .loaded_hash
                .as_ref()
                .expect("BUG: In fast refresh, 'loaded hash' has no hash");

            if !primary_hash.equals(secondary_hash) {
                //both date and size are same
                if primary_file.metadata.size == secondary_file.metadata.size
                    && primary_file.metadata.last_modified == secondary_file.metadata.last_modified
                {
                    self.invalid_hash_index.push(*primary_file_index);
                }
                //only the size is different
                else if primary_file.metadata.size != secondary_file.metadata.size
                    && primary_file.metadata.last_modified == secondary_file.metadata.last_modified
                {
                    self.dirty_potentially_invalid_s_files_index
                        .push(*primary_file_index);
                }
                //only the date is different
                else if primary_file.metadata.size == secondary_file.metadata.size
                    && primary_file.metadata.last_modified != secondary_file.metadata.last_modified
                {
                    self.dirty_potentially_invalid_d_files_index
                        .push(*primary_file_index);
                }
                //both date and size are different
                else {
                    self.dirty_potentially_invalid_sd_files_index
                        .push(*primary_file_index);
                }
            } else {
                self.dirty_valid_files_index.push(*primary_file_index);
            }
        }

        //Cross compare the files that are only in primary and secondary

        let mut primary_new_files_hash_index: HashMap<H, Vec<usize>> = HashMap::new();

        for (index) in only_in_primary_index.iter() {
            let file = &self.primary_ds[*index];
            if let Some(hash) = &file.calculated_hash {
                primary_new_files_hash_index
                    .entry(hash.clone())
                    .or_insert_with(Vec::new)
                    .push(*index);
            } else {
                panic!("File at index {} has no calculated hash", index);
            }
        }

        let mut secondary_files_hash_index: HashMap<H, Vec<usize>> = HashMap::new();

        for (index) in 0..self.secondary_ds.len() {
            let file = &self.secondary_ds[index];
            if let Some(hash) = &file.loaded_hash {
                secondary_files_hash_index
                    .entry(hash.clone())
                    .or_insert_with(Vec::new)
                    .push(index);
            } else {
                panic!("File at index {} has no loaded hash", index);
            }
        }

        let mut found_primary_indexes: HashSet<usize> = HashSet::new();

        for (index) in only_in_secondary_index.iter() {
            let file = &self.secondary_ds[*index];
            if let Some(hash) = &file.loaded_hash {
                if let Some(primary_indexes) = primary_new_files_hash_index.get(hash) {
                    // Update found_primary_indexes
                    found_primary_indexes.extend(primary_indexes.iter().cloned());

                    if let Some((_, secondary_indexes)) = self
                        .crosscheck_primary_to_secondary_found_index
                        .get_mut(hash)
                    {
                        secondary_indexes.push(*index);
                    } else {
                        self.crosscheck_primary_to_secondary_found_index
                            .insert(hash.clone(), (primary_indexes.clone(), vec![*index]));
                    }
                } else {
                    //Push if and only if there are no duplicates in the secondary for this hash
                    //scenario: fileA, fileB, fileC have the same hash. if fileA is not in primary,
                    //but fileB and fileC are, then fileA is an orphan - but since fileB and fileC are
                    //in both primary and secondary, they are not orphans.
                    match secondary_files_hash_index.get(hash) {
                        Some(indexes) => {
                            if indexes.len() == 1 {
                                self.crosscheck_secondary_orphans_index.push(*index);
                            } else {
                                self.crosscheck_secondary_orphan_but_duplicate_index
                                    .push(*index);
                            }
                        }
                        None => {
                            panic!(
                                "BUG: At index {} could not find the Vector collection.",
                                index
                            );
                        }
                    }
                }
            } else {
                panic!("BUG: File at index {} has no loaded hash", index);
            }
        }

        for (index) in only_in_primary_index.iter() {
            if !found_primary_indexes.contains(index) {
                self.crosscheck_primary_orphans_index.push(*index);
            }
        }

        //though it's weird to copy to 'calculated_hash', this ensures that all files being
        //saved into dd have a hash since it's not possible to save into a digest if one of them is missing.
        for (hash, (primary_indexes, _)) in self.crosscheck_primary_to_secondary_found_index.iter()
        {
            for primary_index in primary_indexes.iter() {
                self.primary_ds[*primary_index].calculated_hash = Some(hash.clone());
            }
        }

        //and from the in_both_index
        for (primary_index, secondary_index) in in_both_index.iter() {
            let primary_file = &mut self.primary_ds[*primary_index];
            let secondary_file = &self.secondary_ds[*secondary_index];

            let secondary_hash = secondary_file
                .loaded_hash
                .as_ref()
                .expect("BUG: In fast refresh, 'loaded hash' has no hash");

            primary_file.calculated_hash = Some(secondary_hash.clone());
        }

        //print all primary files that dont have a hash
        for (index, file) in self.primary_ds.iter().enumerate() {
            if file.calculated_hash.is_none() {
                println!("Primary file at index {} has no hash", index);
                println!("Path: {}", file.path.display());
            }
        }

        Ok(())
    }

    fn start_full_refresh(&mut self) -> Result<(), String> {
        todo!()
    }

    // ############################################################################################

    fn print_log_generate(&self) {
        if self.mode != Some(Mode::Generate) {
            println!(
                "{}",
                colorize_txt(TextColor::Red, "BUG: Engine is not in generate mode")
            );
            return;
        }

        //print all the files that failed to hash
        println!("{}", colorize_txt(TextColor::BrightYellow, "######"));
        println!(
            "{}",
            colorize_txt(TextColor::BrightYellow, "Files that failed to hash:")
        );
        println!("{}", colorize_txt(TextColor::BrightYellow, "######"));
        if !self.hashing_error_index.is_empty() {
            for index in self.hashing_error_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }
    }

    fn print_log_validate(&self) {
        if self.mode != Some(Mode::Validate) {
            println!(
                "{}",
                colorize_txt(TextColor::Red, "BUG: Engine is not in validate mode")
            );
            return;
        }

        //print all the files that failed to hash
        if !self.hashing_error_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));
            println!(
                "{}",
                colorize_txt(TextColor::BrightYellow, "Files that failed to hash:")
            );
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));

            for index in self.hashing_error_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the files that have invalid hashes
        if !self.invalid_hash_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));
            println!(
                "{}",
                colorize_txt(TextColor::BrightYellow, "Files that have invalid hashes:")
            );
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));

            for index in self.invalid_hash_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }
    }

    fn print_log_fast_refresh(&self) {
        if self.mode != Some(Mode::FastRefresh) {
            println!(
                "{}",
                colorize_txt(TextColor::Red, "BUG: Engine is not in generate mode")
            );
            return;
        }

        let ok = self.crosscheck_primary_to_secondary_found_index.len()
            + self.dirty_valid_files_index.len();

        let warning = self.dirty_potentially_invalid_d_files_index.len()
            + self.dirty_potentially_invalid_s_files_index.len()
            + self.dirty_potentially_invalid_sd_files_index.len()
            + self.crosscheck_secondary_orphans_index.len()
            + self.crosscheck_primary_orphans_index.len()
            + self.crosscheck_secondary_orphan_but_duplicate_index.len();

        let error = self.invalid_hash_index.len() + self.hashing_error_index.len();

        println!(
            "{}",
            colorize_txt(
                TextColor::BrightYellow,
                &format!(
                    "There are {} successfull operations, {} warnings and {} errors",
                    ok, warning, error
                )
            )
        );

        //print all the files that failed to hash
        if !self.hashing_error_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightRed, "######"));
            println!(
                "{}",
                colorize_txt(TextColor::BrightYellow, "Files that failed to hash:")
            );
            println!("{}", colorize_txt(TextColor::BrightRed, "######"));

            for index in self.hashing_error_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the files that have invalid hashes
        if !self.invalid_hash_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightRed, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightYellow,
                    "Files that have invalid hashes but identical size and last modified date:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightRed, "######"));

            for index in self.invalid_hash_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the files that are potentially invalid due to different size
        if !self.dirty_potentially_invalid_s_files_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightYellow,
                    "Files that have different size and hash:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));

            for index in self.dirty_potentially_invalid_s_files_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the files that are potentially invalid due to different date
        if !self.dirty_potentially_invalid_d_files_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightYellow,
                    "Files that have different last modified date and hash:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));

            for index in self.dirty_potentially_invalid_d_files_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the files that are potentially invalid due to different size and date
        if !self.dirty_potentially_invalid_sd_files_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightYellow,
                    "Files that have different size, last modified date and hash:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightYellow, "######"));

            for index in self.dirty_potentially_invalid_sd_files_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the files that couldn't be crosschecked, only in secondary
        if !self.crosscheck_secondary_orphans_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightMagenta, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightMagenta,
                    "Files that were only found in the digest file:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightMagenta, "######"));

            for index in self.crosscheck_secondary_orphans_index.iter() {
                println!("{}", self.secondary_ds[*index].path.display());
            }
        }

        //print all the files that couldn't be crosschecked, only in primary
        if !self.crosscheck_primary_orphans_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightMagenta, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightMagenta,
                    "Files that were only found on disk:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightMagenta, "######"));

            for index in self.crosscheck_primary_orphans_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the duplicate secondary orphans
        if !self
            .crosscheck_secondary_orphan_but_duplicate_index
            .is_empty()
        {
            println!("{}", colorize_txt(TextColor::BrightMagenta, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightMagenta,
                    "Files that were only found in the digest file and have duplicates in it:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightMagenta, "######"));

            for index in self.crosscheck_secondary_orphan_but_duplicate_index.iter() {
                println!("{}", self.secondary_ds[*index].path.display());
            }
        }

        //print all the files that are validated
        if !self.dirty_valid_files_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightGreen, "######"));
            println!(
                "{}",
                colorize_txt(
                    TextColor::BrightGreen,
                    "Files that have different date or last modified date, but identical hashes:"
                )
            );
            println!("{}", colorize_txt(TextColor::BrightGreen, "######"));

            for index in self.dirty_valid_files_index.iter() {
                println!("{}", self.primary_ds[*index].path.display());
            }
        }

        //print all the files that are crosschecked
        if !self.crosscheck_primary_to_secondary_found_index.is_empty() {
            println!("{}", colorize_txt(TextColor::BrightGreen, "######"));
            println!(
                "{}",
                colorize_txt(TextColor::BrightGreen, "Files that were found:")
            );
            println!("{}", colorize_txt(TextColor::BrightGreen, "######"));

            for (_, (primary_indexes, secondary_indexes)) in
                self.crosscheck_primary_to_secondary_found_index.iter()
            {
                println!("{}", colorize_txt(TextColor::BrightGreen, "------"));
                for index in primary_indexes.iter() {
                    println!("{}", self.primary_ds[*index].path.display());
                }
                println!(
                    "{}",
                    colorize_txt(TextColor::BrightGreen, "@ V From digest:")
                );
                for index in secondary_indexes.iter() {
                    println!("{}", self.secondary_ds[*index].path.display());
                }
            }
        }
        println!("{}", colorize_txt(TextColor::BrightGreen, "------"));
    }

    fn print_log_full_refresh(&self) {
        todo!()
    }

    fn event_count_generate(&self) -> usize {
        self.hashing_error_index.len()
    }

    fn event_count_validate(&self) -> usize {
        self.hashing_error_index.len() + self.invalid_hash_index.len()
    }

    fn event_count_fast_refresh(&self) -> usize {
        self.hashing_error_index.len()
            + self.invalid_hash_index.len()
            + self.dirty_potentially_invalid_d_files_index.len()
            + self.dirty_potentially_invalid_s_files_index.len()
            + self.dirty_potentially_invalid_sd_files_index.len()
            + self.dirty_valid_files_index.len()
            + self.crosscheck_secondary_orphans_index.len()
            + self.crosscheck_primary_orphans_index.len()
            + self.crosscheck_secondary_orphan_but_duplicate_index.len()
            + self.crosscheck_primary_to_secondary_found_index.len()
    }

    fn event_count_full_refresh(&self) -> usize {
        todo!()
    }
}

//todo: filter out files based on date created and digest creation
