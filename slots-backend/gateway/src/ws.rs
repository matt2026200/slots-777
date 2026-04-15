use axum::extract::ws::{WebSocket, Message};
use futures::StreamExt;
use common::{redis::*, model::*};
use serde_json::json;
use redis::aio::MultiplexedConnection;

pub async fn handle_ws(mut socket: WebSocket, token: String) {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut redis: MultiplexedConnection =
        client.get_multiplexed_async_connection().await.unwrap();

    let uid = match get_uid(&mut redis, &token).await {
        Some(u) => u,
        None => return,
    };

    while let Some(Ok(Message::Text(text))) = socket.next().await {
        let req: WsRequest = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let bet = req.data["bet"].as_i64().unwrap_or(0) as i32;

        let result = crate::grpc_client::spin(
            uid.clone(),
            bet,
            req.request_id.clone(),
        )
        .await;

        match result {
            Ok((result, win)) => {
                let _ = socket.send(Message::Text(
                    json!({
                        "request_id": req.request_id,
                        "code": 0,
                        "msg": "ok",
                        "data": {
                            "result": result,
                            "win": win
                        }
                    })
                    .to_string()
                    .into(),
                )).await;
            }
            Err(msg) => {
                let _ = socket.send(Message::Text(
                    json!({
                        "request_id": req.request_id,
                        "code": 1001,
                        "msg": msg
                    })
                    .to_string()
                    .into(),
                )).await;
            }
        }
    }
}