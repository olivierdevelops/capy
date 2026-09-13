//! Port of `infra/file_reader.go`.

use crate::gopath;
use std::fs;

#[derive(Debug, Clone, Copy, Default)]
pub struct FileReader;

impl FileReader {
    pub fn read(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| gopath::io_error("open", path, &e))
    }

    pub fn write(&self, path: &str, content: &str) -> Result<(), String> {
        fs::write(path, content.as_bytes()).map_err(|e| gopath::io_error("open", path, &e))
    }
}
