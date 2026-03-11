use axum::{
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .nest_service("/static", ServeDir::new("crates/web/static"))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🌐 Web app running on http://{}", addr);
    println!("📁 Serving files from crates/web/");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Result<Html<String>, StatusCode> {
    match tokio::fs::read_to_string("crates/web/index.html").await {
        Ok(content) => Ok(Html(content)),
        Err(_) => match tokio::fs::read_to_string("index.html").await {
            Ok(content) => Ok(Html(content)),
            Err(_) => Err(StatusCode::NOT_FOUND),
        },
    }
}
