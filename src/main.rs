mod error;
use axum::{ http::Method, routing::get, Router, extract::Extension };
use tower_http::cors::{ Any, CorsLayer };

use async_graphql_axum::{ GraphQLRequest, GraphQLResponse };

use async_graphql::{ Context, EmptySubscription, Object, Schema, SimpleObject };

use serde::Serialize;

use std::sync::{ Arc, Mutex };


// App state, replace with dynamo db connection
struct AppState {
    next_user_id: Mutex<u64>,
}

// structs

#[derive(SimpleObject, Serialize)]
struct User {
    id: u64,
    username: String,
}

// GraphQL Schema
//  Query root
#[derive(Debug)]
struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn sup(&self) -> String {
        "sup, crabs?".to_string()
    }
}

// Mutation root
#[derive(Debug)]
struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(&self, ctx: &Context<'_>, username: String) -> User {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let mut id_guard = state.next_user_id.lock().unwrap();
        let id = *id_guard;
        *id_guard += 1;

        User {
            id,
            username,
        }
    }
}

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
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Define app state
    // Replace with db connection
    let state = Arc::new(AppState {
        next_user_id: Mutex::new(1337),
    });

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).data(state).finish();

    // Configure cors
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Initialize axum router and add route endpoints
    let app = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);

    // Run app with hyper, listen globally on port 3000
    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
