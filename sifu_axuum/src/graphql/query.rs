use async_graphql::{Context, Object, Result, SimpleObject};
use sqlx::PgPool;

use crate::middleware::Claims;

#[derive(SimpleObject, Clone)]
pub struct WordHistory {
    pub id: String,
    pub word: String,
    pub time: String,
}

impl From<my_type::model::dict::WordHistory> for WordHistory {
    fn from(value: my_type::model::dict::WordHistory) -> Self {
        Self {
            id: value.id.to_string(),
            word: value.word,
            time: value.time.to_rfc3339(),
        }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello, GraphQL!".to_string()
    }

    async fn version(&self) -> String {
        "0.1.0".to_string()
    }

    async fn word_histories(&self, ctx: &Context<'_>) -> Result<Vec<WordHistory>> {
        let pool = ctx.data::<PgPool>()?;
        let claims = ctx.data::<Claims>()?;
        let user_id = uuid::Uuid::parse_str(&claims.id)
            .map_err(|e| async_graphql::Error::new(format!("无效的用户 ID: {}", e)))?;

        let histories: Vec<my_type::model::dict::WordHistory> =
            sqlx::query_as("SELECT * FROM word_histories WHERE user_id = $1 ORDER BY time DESC")
                .bind(user_id)
                .fetch_all(pool)
                .await
                .map_err(|e| async_graphql::Error::new(format!("查询失败: {}", e)))?;

        Ok(histories.into_iter().map(WordHistory::from).collect())
    }
}
