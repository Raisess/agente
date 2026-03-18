use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub username: String,
    pub hostname: String,
    pub directory: String,
}

impl Session {
    pub fn new(directory: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            started_at: Utc::now(),
            updated_at: Utc::now(),
            username: whoami::username().expect("should get the username"),
            hostname: whoami::hostname().expect("should get the hostname"),
            directory,
        }
    }

    pub fn update(&mut self) -> () {
        self.updated_at = Utc::now();
    }
}
