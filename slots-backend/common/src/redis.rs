use redis::{AsyncCommands, aio::MultiplexedConnection};

/// 设置 token 和 uid，过期时间 3600 秒
pub async fn set_token(conn: &mut MultiplexedConnection, token: &str, uid: &str) {
    let _: () = conn
        .set_ex(format!("token:{}", token), uid, 3600)
        .await
        .unwrap();
}

/// 根据 token 获取 uid
pub async fn get_uid(conn: &mut MultiplexedConnection, token: &str) -> Option<String> {
    conn.get(format!("token:{}", token)).await.ok()
}

/// 获取用户余额，找不到返回 0
pub async fn get_balance(conn: &mut MultiplexedConnection, uid: &str) -> i32 {
    conn.get(format!("bal:{}", uid)).await.unwrap_or(0)
}

/// 增减用户余额
pub async fn add_balance(conn: &mut MultiplexedConnection, uid: &str, v: i32) {
    let _: () = conn.incr(format!("bal:{}", uid), v).await.unwrap();
}