mod models;

use async_graphql::http::GraphiQLSource;
use async_graphql::{Schema, ID, EmptyMutation, EmptySubscription};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Server;
use axum::{Extension, Router};
use http::HeaderMap;
use std::net::Ipv4Addr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use models::{InMemoryBroker, InMemoryDb, MutationRoot, QueryRoot, SubscriptionRoot};

type GraphqlSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

fn get_player_id_from_headers(headers: &HeaderMap) -> Option<ID> {
    headers
        .get("player")
        .and_then(|value| value.to_str().map(ID::from).ok())
}

async fn simple_graphql_handler(
    schema: Extension<GraphqlSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(player_id) = get_player_id_from_headers(&headers) {
        req = req.data(player_id);
    }

    schema.execute(req).await.into()
}

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

fn app() -> Router {
    let in_memory_db = InMemoryDb::default();
    let in_memory_broker = InMemoryBroker::default();
    // TODO use the right operations from model.rs
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .enable_federation()
        .limit_complexity(100)
        .data(in_memory_db)
        .data(in_memory_broker)
        .finish();

    Router::new()
        .route("/", get(graphiql).post(simple_graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(CorsLayer::permissive())
        .layer(ServiceBuilder::new().layer(Extension(schema)))
}

#[tokio::main]
async fn main() {
    let app = app();
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "4005".to_string())
        .parse::<u16>()
        .unwrap();

    println!(
        "Explore this graph at https://studio.apollographql.com/sandbox/explorer?endpoint={}",
        urlencoding::encode(&format!("http://localhost:{port}"))
    );

    Server::bind(&(Ipv4Addr::new(0, 0, 0, 0), port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
