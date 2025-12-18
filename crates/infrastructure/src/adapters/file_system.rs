use agente_domain::ports::io::{Reader, Writer};

#[derive(Default)]
pub struct FileSystem;

impl Reader for FileSystem {
    fn read(&self, path: &str) -> Result<String, std::io::Error> {
        let data = std::fs::read_to_string(path)?;
        Ok(data)
    }
}

impl Writer for FileSystem {
    fn write(&self, path: &str, data: &[u8]) -> Result<(), std::io::Error> {
        std::fs::write(path, data)?;
        Ok(())
    }
}
