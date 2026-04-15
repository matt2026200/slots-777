use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use common::redis::*;
use crate::grpc_client;
use redis::aio::MultiplexedConnection;

#[derive(Deserialize)]
pub struct SpinHttpReq {
    pub token: String,
    pub request_id: String,
    pub bet: i32,
}

#[derive(Serialize)]
pub struct SpinHttpResp {
    pub request_id: String,
    pub code: i32,
    pub msg: String,
    pub result: Option<String>,
    pub win: Option<i32>,
}

pub async fn spin(Json(req): Json<SpinHttpReq>) -> impl IntoResponse {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn: MultiplexedConnection =
        client.get_multiplexed_async_connection().await.unwrap();

    let uid = match get_uid(&mut conn, &req.token).await {
        Some(u) => u,
        None => {
            return Json(SpinHttpResp {
                request_id: req.request_id,
                code: 401,
                msg: "invalid token".into(),
                result: None,
                win: None,
            });
        }
    };

    let result = grpc_client::spin(
        uid.clone(),
        req.bet,
        req.request_id.clone(),
    )
    .await;

    match result {
        Ok((result, win)) => Json(SpinHttpResp {
            request_id: req.request_id,
            code: 0,
            msg: "ok".into(),
            result: Some(result),
            win: Some(win),
        }),
        Err(msg) => Json(SpinHttpResp {
            request_id: req.request_id,
            code: 1001,
            msg,
            result: None,
            win: None,
        }),
    }
}