use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sent_at: DateTime<Utc>,
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn new(session_id: Uuid, role: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            sent_at: Utc::now(),
            role,
            content,
        }
    }
}
