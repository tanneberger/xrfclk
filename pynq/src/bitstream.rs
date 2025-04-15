use std::path::{Path, PathBuf};

pub struct BitStream {
    bitstream_file: PathBuf,
    dtbo: String,
    partial: bool,
    firmware_patch: String,
    timestamp: String,
}

impl BitStream {
    pub fn new(bitstream_file: &Path) -> Self {
        Self {
            bitstream_file: bitstream_file.to_path_buf(),
            dtbo: String::new(),
            partial: false,
            firmware_patch: String::new(),
            timestamp: String::new(),
        }
    }
}
