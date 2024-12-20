use crate::file_rep::file_st::FileSt;
use crate::file_rep::hash_def::HashValue;
use crate::util::console_text_formatter::{colorize_txt, TextColor};
use lazy_static::lazy_static;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::io::{stderr, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

lazy_static! {
    static ref INTERRUPT_FLAG: Arc<AtomicBool> = {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        ctrlc::set_handler(move || {
            flag_clone.store(true, Ordering::SeqCst);
            eprintln!("\r\nInterrupting... Please wait for current operations to complete...");
        })
        .expect("Error setting Ctrl-C handler");

        flag
    };
}

struct ProgressTracker {
    last_update: Instant,
    update_interval: Duration,
}

impl ProgressTracker {
    fn new(update_interval_ms: u64) -> Self {
        Self {
            last_update: Instant::now(),
            update_interval: Duration::from_millis(update_interval_ms),
        }
    }

    fn should_update(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.last_update = now;
            true
        } else {
            false
        }
    }
}

pub fn calculate_hashes<'a, H, I>(
    files_iter: I,
    small_threads: usize,
    large_threads: usize,
    size_threshold: u64,
    indexes_to_hash: Option<Vec<usize>>,
) -> Result<Option<Vec<usize>>, String>
where
    H: HashValue + Send + Sync + 'a,
    I: IntoIterator<Item = &'a mut FileSt<H>>,
{
    let interrupt_flag = INTERRUPT_FLAG.clone();

    let small_pool = ThreadPoolBuilder::new()
        .num_threads(small_threads)
        .build()
        .map_err(|e| e.to_string())?;

    let large_pool = ThreadPoolBuilder::new()
        .num_threads(large_threads)
        .build()
        .map_err(|e| e.to_string())?;

    let files: Vec<_> = files_iter.into_iter().collect();

    let indexes_set = indexes_to_hash.map(|indexes| {
        indexes
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
    });

    let mut small_files = Vec::new();
    let mut large_files = Vec::new();
    let mut small_indexes = Vec::new();
    let mut large_indexes = Vec::new();

    for (index, file) in files.into_iter().enumerate() {
        if let Some(ref indexes) = indexes_set {
            if !indexes.contains(&index) {
                continue;
            }
        }

        if file.metadata.size <= size_threshold {
            small_indexes.push(index);
            small_files.push(file);
        } else {
            large_indexes.push(index);
            large_files.push(file);
        }
    }

    let total_small = small_files.len();
    let total_large = large_files.len();

    let remaining_small = Arc::new(AtomicUsize::new(total_small));
    let remaining_large = Arc::new(AtomicUsize::new(total_large));
    let failed_indexes = Arc::new(Mutex::new(Vec::new()));
    let progress_tracker = Arc::new(Mutex::new(ProgressTracker::new(100)));
    let stopwatch = Instant::now();

    //Small files
    let remaining_small_clone = remaining_small.clone();
    let tracker_clone = progress_tracker.clone();
    let failed_indexes_clone = failed_indexes.clone();
    let interrupt_flag_clone = interrupt_flag.clone();

    small_pool
        .install(|| {
            small_files
                .par_iter_mut()
                .enumerate()
                .try_for_each(|(local_index, file)| {
                    if interrupt_flag_clone.load(Ordering::SeqCst) {
                        return Err(());
                    }

                    if let Err(_) = file.calc_hash() {
                        if let Ok(mut failed) = failed_indexes_clone.lock() {
                            failed.push(small_indexes[local_index]);
                        }
                    }

                    let remaining = remaining_small_clone.fetch_sub(1, Ordering::Relaxed);
                    if let Ok(mut tracker) = tracker_clone.lock() {
                        if tracker.should_update() {
                            print_progress(remaining - 1, true);
                        }
                    }
                    Ok(())
                })
        })
        .map_err(|_| "Error hashing small files".to_string())?;

    if total_small > 0 && failed_indexes.lock().unwrap().len() == total_small {
        return Err("Failed to hash any small files".to_string());
    }

    if interrupt_flag.load(Ordering::SeqCst) {
        return Err("Interrupted".to_string());
    }

    //Large files
    let remaining_large_clone = remaining_large.clone();
    let tracker_clone = progress_tracker;
    let failed_indexes_clone = failed_indexes.clone();
    let interrupt_flag_clone = interrupt_flag.clone();

    large_pool
        .install(|| {
            large_files
                .par_iter_mut()
                .enumerate()
                .try_for_each(|(local_index, file)| {
                    if interrupt_flag_clone.load(Ordering::SeqCst) {
                        return Err(());
                    }

                    if let Err(_) = file.calc_hash() {
                        if let Ok(mut failed) = failed_indexes_clone.lock() {
                            failed.push(large_indexes[local_index]);
                        }
                    }

                    let remaining = remaining_large_clone.fetch_sub(1, Ordering::Relaxed);
                    if let Ok(mut tracker) = tracker_clone.lock() {
                        if tracker.should_update() {
                            print_progress(remaining - 1, false);
                        }
                    }
                    Ok(())
                })
        })
        .map_err(|_| "Error hashing large files".to_string())?;

    //Clear the progress line
    eprintln!("\r{}\r", " ".repeat(70));

    let elapsed = stopwatch.elapsed();
    let elapsed_str = format!(
        "Elapsed time for hashing: {:02}:{:02}:{:02}",
        elapsed.as_secs() / 3600,
        elapsed.as_secs() % 3600 / 60,
        elapsed.as_secs() % 60
    );
    eprintln!("{}", elapsed_str);

    let failed = failed_indexes.lock().unwrap();
    if failed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(failed.to_vec()))
    }
}

fn print_progress(remaining: usize, is_small: bool) {
    let indicator = if is_small { "(S: 1/2)" } else { "(L: 2/2)" };
    let remaining_str = format!(
        "{} {} {} ",
        colorize_txt(TextColor::BrightMagenta, &*remaining.to_string()),
        colorize_txt(TextColor::BrightMagenta, "Files remaining"),
        colorize_txt(TextColor::BrightMagenta, indicator)
    );

    let _ = stderr().write_fmt(format_args!("\r{:<50}", remaining_str));
    let _ = stderr().flush();
}
