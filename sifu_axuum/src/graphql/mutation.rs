use async_graphql::Object;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn ping(&self) -> String {
        "pong".to_string()
    }
}
