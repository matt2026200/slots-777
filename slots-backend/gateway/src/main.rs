use axum::{
    Router,
    routing::{get, post},
    extract::{ws::WebSocketUpgrade, Query},
};
use serde::Deserialize;
use tokio::net::TcpListener;

mod ws;
mod grpc_client;
mod http;

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(q): Query<WsQuery>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| ws::handle_ws(socket, q.token))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/spin", post(http::spin))
        .route("/health", get(|| async { "ok" }));

    let listener = TcpListener::bind("0.0.0.0:5000").await.unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}
