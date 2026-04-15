use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use tokio_postgres::NoTls;
use serde_json::Value as JsonValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ================= Redis =================
    let client = Client::open("redis://127.0.0.1/0")?;
    let mut conn: MultiplexedConnection =
        client.get_multiplexed_async_connection().await?;

    // ================= Postgres =================
    let (pg_client, connection) = tokio_postgres::connect(
        "host=127.0.0.1 user=mattlee password=123456 dbname=slots_db",
        NoTls,
    )
    .await?;

    // 后台维护 PG 连接
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("❌ Postgres connection error: {}", e);
        }
    });

    println!("🚀 mq2db-service started (Redis Stream)");

    // ================= Stream =================
    let stream_key = "wallet_stream";
    let mut last_id = "0-0".to_string();

    loop {
        let reply: redis::streams::StreamReadReply = redis::cmd("XREAD")
            .arg("BLOCK").arg(5000)   // 阻塞5秒
            .arg("COUNT").arg(10)     // 每次最多10条
            .arg("STREAMS")
            .arg(stream_key)
            .arg(&last_id)
            .query_async(&mut conn)
            .await
            .unwrap_or(redis::streams::StreamReadReply { keys: vec![] });

        if reply.keys.is_empty() {
            continue;
        }

        for stream in reply.keys {
            for item in stream.ids {
                let current_id = item.id.clone();

                // ===== 解析 msg =====
                let msg_str: String = match item.map.get("msg") {
                    Some(val) => match redis::from_redis_value::<String>(val.clone()) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("❌ parse redis value error: {}", e);
                            last_id = current_id;
                            continue;
                        }
                    },
                    None => {
                        last_id = current_id;
                        continue;
                    }
                };

                println!("📩 收到 MQ: {}", msg_str);

                // ===== JSON =====
                let v: JsonValue = match serde_json::from_str(&msg_str) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("❌ JSON parse error: {}", e);
                        last_id = current_id;
                        continue;
                    }
                };

                let request_id = v["request_id"].as_str().unwrap_or("");
                let user_id = v["user_id"].as_str().unwrap_or("");
                let typ = v["type"].as_str().unwrap_or("");

                // ✅ BIGINT 对应 i64
                let amount: i64 = v["amount"].as_i64().unwrap_or(0);
                let balance: i64 = v["balance"].as_i64().unwrap_or(0);

                // ===== 写 wallet_tx（让 PG 自动填 created_at）=====
                if let Err(e) = pg_client.execute(
                    "INSERT INTO wallet_tx(request_id, user_id, type, amount)
                     VALUES ($1, $2, $3, $4)",
                    &[&request_id, &user_id, &typ, &amount],
                ).await {
                    eprintln!("❌ insert wallet_tx error: {}", e);

                    // ⚠️ 防止死循环
                    last_id = current_id;
                    continue;
                }

                // ===== 写 wallet_balance =====
                if let Err(e) = pg_client.execute(
                    "INSERT INTO wallet_balance(user_id, balance, updated_at)
                     VALUES ($1, $2, now())
                     ON CONFLICT (user_id)
                     DO UPDATE SET balance = EXCLUDED.balance, updated_at = now()",
                    &[&user_id, &balance],
                ).await {
                    eprintln!("❌ insert wallet_balance error: {}", e);

                    last_id = current_id;
                    continue;
                }

                println!("✅ DB 写入成功 user={} balance={}", user_id, balance);

                // ✅ 推进 offset
                last_id = current_id;
            }
        }
    }
}
