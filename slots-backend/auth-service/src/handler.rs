use axum::{Json, http::StatusCode};
use redis::AsyncCommands;
use common::redis::*;
use common::jwt::create_token;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginReq {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct LoginResp {
    pub token: String,
}

pub async fn login(
    Json(req): Json<LoginReq>,
) -> Result<Json<LoginResp>, StatusCode> {
    // 1️⃣ 生成 token
    let token = create_token(&req.user_id);

    // 2️⃣ Redis 客户端
    let client = redis::Client::open("redis://127.0.0.1/")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3️⃣ 存 token
    set_token(&mut conn, &token, &req.user_id).await;

    // 4️⃣ 初始化余额（仅在不存在时）
    let _: bool = conn
        .set_nx(format!("bal:{}", req.user_id), 10000)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 5️⃣ 返回 token
    Ok(Json(LoginResp { token }))
}