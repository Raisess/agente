use std::io::Read;

use agente_domain::ports::io::{Reader, Writer};

pub struct FileSystem;

impl FileSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl Reader for FileSystem {
    fn read(&self, path: &str) -> Result<String, std::io::Error> {
        let mut file = std::fs::File::open(path)?;

        let mut data = String::new();
        file.read_to_string(&mut data)?;

        Ok(data)
    }
}

impl Writer for FileSystem {
    fn write(&self, _path: &str, _data: &[u8]) -> Result<(), std::io::Error> {
        todo!()
    }
}
