use std::sync::Arc;

/// Represents a SQL Database connection exposer
pub trait Database<T>: Send + Sync {
    // Exposes the sql database connection
    fn expose(&self) -> Arc<T>;
}
