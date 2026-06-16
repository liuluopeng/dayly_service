use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl BaseEntity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(id: Uuid) -> Self {
        Self {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

crate::impl_display!(self, BaseEntity,
    "ID"       => self.id,
    "创建时间" => self.created_at
);

pub fn generate_id() -> Uuid {
    Uuid::now_v7()
}

pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}
