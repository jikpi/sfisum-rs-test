use std::time::SystemTime;

#[derive(Debug)]
pub struct FileMetadata {
    pub last_modified: SystemTime,
    pub size: u64,
}

impl FileMetadata {
    pub fn new(last_modified: SystemTime, size: u64) -> Self {
        FileMetadata {
            last_modified,
            size,
        }
    }

    pub fn new_from_string<S: AsRef<str>>(input: S) -> Result<Self, &'static str> {
        let s = input.as_ref();

        let parts: Vec<&str> = s.split(", ").collect();
        if parts.len() != 2 {
            return Err("Invalid metadata string format");
        }

        let size = match parts[0].strip_prefix("Size: ").and_then(|s| s.parse().ok()) {
            Some(size) => size,
            None => return Err("Invalid size format"),
        };

        let secs = match parts[1]
            .strip_prefix("Last modified: ")
            .and_then(|s| s.parse().ok())
        {
            Some(secs) => secs,
            None => return Err("Invalid timestamp format"),
        };

        let last_modified = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);

        Ok(FileMetadata {
            last_modified,
            size,
        })
    }

    pub fn to_string(&self) -> String {
        format!(
            "Size: {:?}, Last modified: {:?}",
            self.size,
            self.last_modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
    }
}
