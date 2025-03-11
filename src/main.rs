use axum::{ routing::{ get, post }, http::StatusCode, Json, Router };
use serde::{ Deserialize, Serialize };

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize axum router and add route endpoints
    let app = Router::new().route("/", get(root)).route("/users", post(create_user));

    // Run app with hyper, listen globally on port 3000
    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// Basic handler that responds with a static string
async fn root() -> &'static str {
    "sup, crabs?"
}

async fn create_user(
    // this argument tells axum to parse the request body
    //  as json into a 'CreateUser' type
    Json(payload): Json<CreateUser>
) -> (StatusCode, Json<User>) {
    // application logic
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // Converts to a JSON response with a status code o 201 created
    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
