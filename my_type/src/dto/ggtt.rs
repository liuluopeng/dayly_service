use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SearchRequest {
    pub search: String,
}