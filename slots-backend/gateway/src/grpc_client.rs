use proto::game::{game_client::GameClient, SpinRequest};
use tonic::Request;

pub async fn spin(
    uid: String,
    bet: i32,
    request_id: String,
) -> Result<(String, i32), String> {
    let mut client = GameClient::connect("http://127.0.0.1:50051")
        .await
        .map_err(|e| e.to_string())?;

    let resp = client
        .spin(Request::new(SpinRequest {
            request_id,
            user_id: uid,
            bet,
        }))
        .await;

    match resp {
        Ok(r) => {
            let r = r.into_inner();
            Ok((r.result, r.win))
        }
        Err(status) => Err(status.message().to_string()),
    }
}
