use async_graphql::{EmptySubscription, Schema};

pub mod handlers;
pub mod mutation;
pub mod query;

pub use handlers::{graphql, graphql_playground};
pub use mutation::MutationRoot;
pub use query::QueryRoot;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
