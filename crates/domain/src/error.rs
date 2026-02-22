/// Generic error wrapper for trait implementations
#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error,
{
    fn from(value: E) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
