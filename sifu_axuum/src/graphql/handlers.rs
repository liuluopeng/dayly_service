use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Extension;
use axum::response::Html;

use crate::graphql::AppSchema;
use crate::middleware::Claims;

pub async fn graphql_playground() -> Html<String> {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

pub async fn graphql(
    claims: Claims,
    Extension(schema): Extension<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let request = req.into_inner().data(claims);
    schema.execute(request).await.into()
}
