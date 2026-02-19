pub mod events;
pub mod loaders;
pub mod mutation;
pub mod query;
pub mod subscription;
pub mod types;

use async_graphql::{EmptyMutation, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::extract::State;
use axum::{Router, routing::get};

use crate::auth::MaybeAuthUser;
use crate::state::AppState;

use loaders::RequestLoaders;
use query::QueryRoot;
use subscription::SubscriptionRoot;

pub type GlintSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

/// Build the full schema with AppState injected. Subscriptions (WS) and queries
/// both access AppState via `ctx.data::<AppState>()` — injecting it at schema
/// level ensures it's available on every execution path.
pub fn build_schema(state: AppState) -> GlintSchema {
    Schema::build(QueryRoot::default(), EmptyMutation, SubscriptionRoot)
        .data(state)
        .limit_depth(15)
        .limit_complexity(200)
        .finish()
}

/// Build the schema without runtime data — only usable for SDL export in tests.
pub fn build_schema_for_sdl() -> GlintSchema {
    Schema::build(QueryRoot::default(), EmptyMutation, SubscriptionRoot)
        .limit_depth(15)
        .limit_complexity(200)
        .finish()
}

pub fn router(schema: GlintSchema) -> Router<AppState> {
    Router::new()
        .route("/graphql", get(graphql_handler).post(graphql_handler))
        .route_service("/graphql/ws", GraphQLSubscription::new(schema.clone()))
        .layer(axum::Extension(schema))
}

async fn graphql_handler(
    schema: axum::extract::Extension<GlintSchema>,
    State(state): State<AppState>,
    _auth: MaybeAuthUser,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let loaders = RequestLoaders::new(state.db().clone());

    let mut req = req.into_inner();
    req = req.data(loaders);

    schema.execute(req).await.into()
}
