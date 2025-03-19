use aws_sdk_dynamodb::Client;
use axum::{ http::Method, routing::get, Router, extract::Extension };
use error::AppError;
use schema::{ MutationRoot, QueryRoot };
use tower::builder::ServiceBuilder;
use tower_http::{ compression::CompressionLayer, cors::{ Any, CorsLayer } };

use async_graphql_axum::{ GraphQLRequest, GraphQLResponse };

use async_graphql::{ Context, EmptySubscription, Object, Schema, SimpleObject };

use serde::Serialize;
use tracing::{ warn, error };

use std::sync::{ Arc, Mutex };

mod schema;
mod error;
mod db;
mod models;

// App state, replace with dynamo db connection
#[derive(Clone)]
pub struct AppState {
    db_client: Client,
}

// Success http response struct
#[derive(Debug, Serialize)]
struct SuccessResponse {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct FailureResponse {
    pub body: String,
}

// Implement Display for FailureResponse
impl std::fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

// Implement Error trait for FailureResponse
impl std::error::Error for FailureResponse {}
// Handler for graphql requests
async fn graphql_handler(
    Extension(schema): Extension<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
    req: GraphQLRequest
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// Handler for graphql playground
async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() {
    // Initialize tracing with detailed configuration
    tracing_subscriber
        ::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .init();

    tracing::info!("Starting up UW Pantry service");

    // Create db client
    let db_client = match db::local::setup_local_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Fatal error during startup: {}", e);
            std::process::exit(1);
        }
    };

    db::init::ensure_tables_exist(&db_client).await.unwrap();

    // Define app state
    // Replace with db connection
    // let state = Arc::new(AppState {
    //     db_client,
    // });

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db_client.clone())
        .finish();

    // Configure cors
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Initialize axum router and add route endpoints
    let app = Router::new().route("/graphql", get(graphql_playground).post(graphql_handler));

    let app = app.layer(
        ServiceBuilder::new()
            .layer(CompressionLayer::new().gzip(true).deflate(true).br(true))
            .layer(Extension(db_client))
            .layer(Extension(schema))
            .layer(cors)
    );

    // Run app with hyper, listen globally on port 3000
    let listener = match tokio::net::TcpListener::bind(&"0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Fatal error during startup: {}", e);
            std::process::exit(1);
        }
    };
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        eprintln!("Fatal error during startup: {}", e);
        std::process::exit(1);
    });
}
