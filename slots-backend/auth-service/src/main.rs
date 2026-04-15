use axum::{routing::post, Router};
mod handler;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/login", post(handler::login));
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
