/// Reprensents a read operation, can be from io, network, etc.
pub trait Reader: Send + Sync {
    /// Reads from a specific path, url, etc.
    ///
    /// @param path - Destination target
    fn read(&self, path: &str) -> Result<String, std::io::Error>;
}

/// Writes to a specific path, that can be io, network, etc.
pub trait Writer: Send + Sync {
    /// Writes data to the path.
    ///
    /// @param path - Destination target
    /// @param data - Bytes to write
    fn write(&self, path: &str, data: &[u8]) -> Result<(), std::io::Error>;
}
