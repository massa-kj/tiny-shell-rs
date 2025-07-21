use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};

pub struct HistoryManager {
    pub entries: Vec<String>,
    pub max_len: usize,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum HistoryMode {
    AllowDuplicates,
    DisallowDuplicates,
}

impl HistoryManager {
    // Load from history file
    pub fn load(path: &str, max_len: usize) -> std::io::Result<Self> {
        let file = File::open(path);
        let mut entries = Vec::new();
        if let Ok(f) = file {
            let reader = BufReader::new(f);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        entries.push(line);
                    }
                }
            }
        }
        // Truncate old history to keep max_len
        if entries.len() > max_len {
            let start = entries.len() - max_len;
            entries = entries[start..].to_vec();
        }
        Ok(Self {
            entries,
            max_len,
            file_path: Some(path.to_string()),
        })
    }

    // Save history
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
            for line in &self.entries {
                writeln!(file, "{}", line)?;
            }
        }
        Ok(())
    }

    // Add a command to history
    pub fn add(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        // Do not add if it's the same as the previous entry
        if self.entries.last().map_or(false, |last| last == trimmed) {
            return;
        }
        self.entries.push(trimmed.to_string());
        // Remove oldest entries if exceeding the limit
        if self.entries.len() > self.max_len {
            self.entries.remove(0);
        }
    }

    // Get the history list (read-only)
    pub fn list(&self) -> &[String] {
        &self.entries
    }

    // Get the nth history entry
    pub fn get(&self, idx: usize) -> Option<&str> {
        self.entries.get(idx).map(|s| s.as_str())
    }

    // Number of history entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    // Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    // Get the latest history entry (the last entered command)
    pub fn last(&self) -> Option<&str> {
        self.entries.last().map(|s| s.as_str())
    }
}

