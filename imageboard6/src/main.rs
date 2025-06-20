mod handlers;
mod models;
mod templates;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Initialize the SQLite database
    models::init_db().expect("Failed to initialize database");

    let app = Router::new()
        .route("/", get(handlers::board))
        .route("/page/{page}", get(handlers::board_page))
        .route("/thread/{id}", get(handlers::thread_view))
        .route("/new", post(handlers::new_thread))
        .route("/reply/{id}", post(handlers::reply))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080)); // Port 8080
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
