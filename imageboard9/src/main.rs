mod handlers;
mod models;
mod templates;
mod boards;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    models::init_db();

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(handlers::landing_page))
        .route("/{board}/", get(handlers::board_page))
        .route("/{board}/page/{page}", get(handlers::board_page_with_page))
        .route("/{board}/new", post(handlers::new_thread))
        .route("/{board}/thread/{id}", get(handlers::thread_view))
        .route("/{board}/reply/{id}", post(handlers::reply))
        .nest_service("/uploads", ServeDir::new("uploads"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
