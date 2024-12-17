use crate::util::console_text_formatter::{colorize_txt, TextColor};

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
            5) Exit\n"
            );

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("{}", colorize_txt(TextColor::Red, "Invalid input. Please enter a number.\n"));
                continue;
            }

            let input = match input.trim().parse::<u8>() {
                Ok(num) => num,
                Err(_) => {
                    println!("{}", colorize_txt(TextColor::Red, "Invalid input. Please enter a number.\n"));
                    continue;
                }
            };

            if input == 5 {
                break;
            }
        }
    }
}
