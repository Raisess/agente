/// Reprensents a read operation, can be from io, network, etc.
pub trait Reader: Send + Sync {
    /// Reads from a specific path, url, etc.
    fn read(&self, path: &str) -> Result<String, std::io::Error>;
}

/// Writes to a specific path, that can be io, network, etc.
pub trait Writer: Send + Sync {
    /// Writes data to the path.
    fn write(&self, path: &str, data: &[u8]) -> Result<(), std::io::Error>;
}

pub enum ExecutorArgument {
    Arg(String),
    Flag((String, String)),
}

/// Represents a host machine command execution.
pub trait Executor: Send + Sync {
    /// Execute a command on the host machine.
    fn exec(
        &self,
        cmd: &str,
        args: Vec<ExecutorArgument>,
        envs: Vec<(String, String)>,
    ) -> Result<String, std::io::Error>;
}
