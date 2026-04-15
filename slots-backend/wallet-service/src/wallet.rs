use redis::{aio::MultiplexedConnection, AsyncCommands, Client, Script, Value};
use redis::streams::{StreamReadOptions, StreamReadReply};
use serde_json::json;

/// Lua 脚本：扣钱
const LUA_DEDUCT: &str = r#"
local bal_key = KEYS[1]
local amount = tonumber(ARGV[1])
local current = tonumber(redis.call("get", bal_key) or "0")
if current < amount then
    return -1
end
local new = redis.call("incrby", bal_key, -amount)
return new
"#;

/// Lua 脚本：加钱
const LUA_ADD: &str = r#"
local bal_key = KEYS[1]
local amount = tonumber(ARGV[1])
local new = redis.call("incrby", bal_key, amount)
return new
"#;

/// 获取 MultiplexedConnection（所有操作共用）
async fn get_conn() -> Result<MultiplexedConnection, String> {
    let client = Client::open("redis://127.0.0.1/0")
        .map_err(|e| format!("Redis open error: {}", e))?;
    client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Redis connect error: {}", e))
}

/// MQ 写入封装（Redis Stream 方案）
async fn push_mq(msg: &serde_json::Value) -> Result<String, String> {
    let mut conn = get_conn().await?;
    let key = "wallet_stream";

    let msg_str = msg.to_string();

    // XADD 写入 Stream
    let stream_id: String = conn
        .xadd(key, "*", &[("msg", msg_str)])
        .await
        .map_err(|e| format!("XADD error: {}", e))?;

    println!("✅ MQ push success, stream_id={}", stream_id);
    Ok(stream_id)
}

/// 扣钱 + 写 MQ
pub async fn deduct_balance(user_id: &str, amount: i64, request_id: &str) -> Result<i64, String> {
    let mut conn = get_conn().await?;
    let bal_key = format!("bal:{}", user_id);
    println!("👉 [DEDUCT] user={} amount={}", user_id, amount);

    let new_bal: i64 = Script::new(LUA_DEDUCT)
        .key(&bal_key)
        .arg(amount)
        .invoke_async(&mut conn)
        .await
        .map_err(|e| format!("Lua deduct error: {}", e))?;

    if new_bal < 0 {
        println!("❌ [DEDUCT] insufficient balance");
        return Err("insufficient balance".into());
    }

    let msg = json!({
        "type": "bet",
        "user_id": user_id,
        "request_id": request_id,
        "amount": amount,
        "balance": new_bal
    });

    println!("👉 pushing MQ: {}", msg);
    push_mq(&msg).await?;

    println!("✅ deduct success, balance={}", new_bal);
    Ok(new_bal)
}

/// 加钱 + 写 MQ
pub async fn add_balance(user_id: &str, amount: i64, request_id: &str) -> Result<i64, String> {
    let mut conn = get_conn().await?;
    let bal_key = format!("bal:{}", user_id);
    println!("👉 [ADD] user={} amount={}", user_id, amount);

    let new_bal: i64 = Script::new(LUA_ADD)
        .key(&bal_key)
        .arg(amount)
        .invoke_async(&mut conn)
        .await
        .map_err(|e| format!("Lua add error: {}", e))?;

    let msg = json!({
        "type": "win",
        "user_id": user_id,
        "request_id": request_id,
        "amount": amount,
        "balance": new_bal
    });

    println!("👉 pushing MQ: {}", msg);
    push_mq(&msg).await?;

    println!("✅ add success, balance={}", new_bal);
    Ok(new_bal)
}