use crate::engine::dd_file_rw::parse_dd_hash_type;
use crate::engine::engine::EngineAny;
use crate::engine::engine_factory::{create_engine, dd_filename_to_hash_type};
use crate::file_rep::hash_def::HashType;
use crate::util::console_text_formatter::{colorize_txt, TextColor};
use std::path::PathBuf;

pub struct Sfisum {}

impl Sfisum {
    pub fn new() -> Sfisum {
        Sfisum {}
    }
    pub fn launch_cui(&self) {
        println!("{}", colorize_txt(TextColor::Cyan, "Welcome to sfisum.\n"));
        loop {
            println!(
                "Select option:\n\
            1) Generate Directory Digest\n\
            2) Validate Directory Digest\n\
            3) Fast Refresh Directory Digest\n\
            4) Full Refresh Directory Digest\n\
            5) Find duplicates\n\
            6) Exit\n"
            );

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!(
                    "{}",
                    colorize_txt(TextColor::Red, "Invalid input. Please enter a number.\n")
                );
                continue;
            }

            let input = match input.trim().parse::<u8>() {
                Ok(num) => num,
                Err(_) => {
                    println!(
                        "{}",
                        colorize_txt(TextColor::Red, "Invalid input. Please enter a number.\n")
                    );
                    continue;
                }
            };

            if input == 6 {
                break;
            }

            match input {
                1 => self.generate_cui(),
                2 => self.validate_cui(),
                3 => self.refresh_cui(true),
                4 => self.refresh_cui(false),
                5 => self.find_duplicates_cui(),
                _ => println!(
                    "{}",
                    colorize_txt(
                        TextColor::Red,
                        "Invalid input. Please enter a valid number.\n"
                    )
                ),
            }
        }
    }

    fn make_engine_from_dd_file_path(&self, dd_file_path: &str) -> Option<Box<dyn EngineAny>> {
        let inferred_hash_type = match dd_filename_to_hash_type(dd_file_path) {
            Some(hash_type) => hash_type,
            None => match parse_dd_hash_type(&PathBuf::from(dd_file_path)) {
                Some(hash_type) => hash_type,
                None => {
                    println!(
                        "{}",
                        colorize_txt(TextColor::Red, "Cannot parse digest file hash type.\n")
                    );
                    return None;
                }
            },
        };

        let engine: Box<dyn EngineAny> = create_engine(inferred_hash_type);
        Some(engine)
    }

    fn generate_cui(&self) {
        println!("Enter the path to the directory you want to generate a digest for:");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!(
                "{}",
                colorize_txt(
                    TextColor::Red,
                    "Invalid input. Please enter a valid path.\n"
                )
            );
            return;
        }
        let path = input.trim();

        println!("Path loaded. Press enter to continue.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let mut engine: Box<dyn EngineAny> = create_engine(HashType::MD5);
        engine.set_paths(PathBuf::new(), path.into());

        match engine.start_generate() {
            Ok(_) => println!(
                "{}",
                colorize_txt(TextColor::Green, "Digest generated successfully.\n")
            ),
            Err(e) => {
                println!(
                    "{}",
                    colorize_txt(TextColor::Red, &format!("Error: {}\n", e))
                );
                return;
            }
        }

        let event_count = engine.event_count_generate();
        if event_count > 0 {
            'discard_block: {
                println!(
                    "{}",
                    colorize_txt(
                        TextColor::BrightBlue,
                        &format!("There are {} events that occurred during hashing. Press enter to view them, or 'd' to discard.\n", event_count)
                    )
                );

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                if input.trim() == "d" {
                    break 'discard_block;
                }

                engine.print_log();
            }
        } else {
            println!("No events occurred during hashing.\n");
        }

        println!("{}", colorize_txt(TextColor::BrightBlue, "\n######\n"));

        self.save_digest_file_dialog(&mut engine);
    }

    pub fn validate_cui(&self) {
        println!("Enter the path to the base directory:");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!(
                "{}",
                colorize_txt(
                    TextColor::Red,
                    "Invalid input. Please enter a valid path.\n"
                )
            );
            return;
        }
        let base_dir_path = input.trim();

        println!("Enter the path to the existing digest file:");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!(
                "{}",
                colorize_txt(
                    TextColor::Red,
                    "Invalid input. Please enter a valid path.\n"
                )
            );
            return;
        }

        let digest_path = input.trim();

        println!("Paths loaded. Press enter to continue.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let mut engine: Box<dyn EngineAny> = match self.make_engine_from_dd_file_path(digest_path) {
            Some(engine) => engine,
            None => return,
        };

        engine.set_paths(PathBuf::from(digest_path), PathBuf::from(base_dir_path));

        match engine.start_validate() {
            Ok(_) => println!("{}", colorize_txt(TextColor::Green, "Digest validated.\n")),
            Err(e) => {
                println!(
                    "{}",
                    colorize_txt(TextColor::Red, &format!("Error during validation: {}\n", e))
                );
                return;
            }
        }

        let event_count = engine.event_count_validate();
        if event_count > 0 {
            'discard_block: {
                println!(
                    "{}",
                    colorize_txt(
                        TextColor::BrightBlue,
                        &format!("There are {} events that occurred during validation. Press enter to view them, or 'd' to discard.\n", event_count)
                    )
                );

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                if input.trim() == "d" {
                    break 'discard_block;
                }

                engine.print_log();
            }
        } else {
            println!("No events occurred during validation.\n");
        }

        println!("{}", colorize_txt(TextColor::BrightBlue, "\n######\n"));
    }

    fn refresh_cui(&self, fast: bool) {
        println!("Enter the path to the base directory:");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!(
                "{}",
                colorize_txt(
                    TextColor::Red,
                    "Invalid input. Please enter a valid path.\n"
                )
            );
            return;
        }
        let base_dir_path = input.trim();

        println!("Enter the path to the existing digest file:");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!(
                "{}",
                colorize_txt(
                    TextColor::Red,
                    "Invalid input. Please enter a valid path.\n"
                )
            );
            return;
        }

        let digest_path = input.trim();

        println!("Paths loaded. Press enter to continue.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let mut engine: Box<dyn EngineAny> = match self.make_engine_from_dd_file_path(digest_path) {
            Some(engine) => engine,
            None => return,
        };

        engine.set_paths(PathBuf::from(digest_path), PathBuf::from(base_dir_path));

        let operation_result = if fast {
            engine.start_fast_refresh()
        } else {
            engine.start_full_refresh()
        };

        let result_text = if fast {
            "Fast refresh complete.\n"
        } else {
            "Full refresh complete.\n"
        };

        match operation_result {
            Ok(_) => println!("{}", colorize_txt(TextColor::Green, result_text)),
            Err(e) => {
                println!(
                    "{}",
                    colorize_txt(TextColor::Red, &format!("Error during validation: {}\n", e))
                );
                return;
            }
        }

        let event_count = if fast {
            engine.event_count_fast_refresh()
        } else {
            engine.event_count_full_refresh()
        };

        if event_count > 0 {
            'discard_block: {
                println!(
                    "{}",
                    colorize_txt(
                        TextColor::BrightBlue,
                        &format!("There are {} events that occurred during validation. Press enter to view them, or 'd' to discard.\n", event_count)
                    )
                );

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                if input.trim() == "d" {
                    break 'discard_block;
                }

                engine.print_log();
            }
        } else {
            println!("No events occurred during validation.\n");
        }

        println!("{}", colorize_txt(TextColor::BrightBlue, "\n######\n"));

        self.save_digest_file_dialog(&mut engine);
    }

    fn save_digest_file_dialog(&self, engine: &mut Box<dyn EngineAny>) {
        loop {
            println!("Enter the path to save the digest file: (or press enter to save in the base directory, or 'd' to discard)");
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!(
                    "{}",
                    colorize_txt(
                        TextColor::Red,
                        "Invalid input. Please enter a valid path.\n"
                    )
                );
                return;
            }

            let path = input.trim();

            if path == "d" {
                return;
            }

            match engine.save_dd_file(Some(path.into())) {
                Ok(_) => {
                    println!(
                        "{}",
                        colorize_txt(TextColor::Green, "Digest file saved successfully.\n")
                    );
                    break;
                }
                Err(e) => println!(
                    "{} {}",
                    colorize_txt(TextColor::Red, "Error saving digest file:"),
                    e
                ),
            }
        }
    }

    fn find_duplicates_cui(&self) {
        println!("Enter the path to the existing digest file:");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!(
                "{}",
                colorize_txt(
                    TextColor::Red,
                    "Invalid input. Please enter a valid path.\n"
                )
            );
            return;
        }

        let digest_path = input.trim();

        println!("Path loaded. Press enter to continue.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let mut engine: Box<dyn EngineAny> = match self.make_engine_from_dd_file_path(digest_path) {
            Some(engine) => engine,
            None => return,
        };

        engine.set_paths(PathBuf::from(digest_path), PathBuf::new());

        match engine.start_find_duplicates() {
            Ok(_) => println!("{}", colorize_txt(TextColor::Green, "Digest analyzed.\n")),
            Err(e) => {
                println!(
                    "{}",
                    colorize_txt(TextColor::Red, &format!("Error during analysis: {}\n", e))
                );
                return;
            }
        }

        let event_count = engine.event_count_find_duplicates();
        if event_count > 0 {
            'discard_block: {
                println!(
                    "{}",
                    colorize_txt(
                        TextColor::BrightBlue,
                        &format!("There are {} events that occurred during analysis. Press enter to view them, or 'd' to discard.\n", event_count)
                    )
                );

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                if input.trim() == "d" {
                    break 'discard_block;
                }

                engine.print_log();
            }
        } else {
            println!("No events occurred during analysis.\n");
        }

        println!("{}", colorize_txt(TextColor::BrightBlue, "\n######\n"));
    }
}
