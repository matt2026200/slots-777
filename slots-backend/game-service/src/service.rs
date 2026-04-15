// game-service/src/service.rs

use tonic::{Request, Response, Status};
use proto::game::{game_server::Game, SpinRequest, SpinResponse};

use proto::wallet::wallet_client::WalletClient;
use proto::wallet::{BetRequest, WinRequest};

pub struct GameSvc;
#[tonic::async_trait]
impl Game for GameSvc {
    async fn spin(
        &self,
        req: Request<SpinRequest>,
    ) -> Result<Response<SpinResponse>, Status> {
        let r = req.into_inner();

        // 1️⃣ 调用 wallet-service 扣钱
        let mut wallet = WalletClient::connect("http://127.0.0.1:50052")
            .await
            .map_err(|e| Status::unavailable(format!("wallet service unavailable: {}", e)))?;

        let bet_resp = wallet
            .bet(Request::new(BetRequest {
                user_id: r.user_id.clone(),
                request_id: r.request_id.clone(),
                amount: r.bet as i64,
            }))
            .await
            .map_err(|e| Status::internal(format!("wallet bet failed: {}", e)))?
            .into_inner();

        if bet_resp.code != 0 {
            // 扣钱失败，直接返回 gRPC 错误
            return Err(Status::failed_precondition(bet_resp.msg.as_str()));
        }

        // 2️⃣ 游戏 spin 逻辑
        let (result, win) = crate::logic::spin(r.bet);

        // 3️⃣ 若赢 → 调用 wallet-service 加钱
        if win > 0 {
            let _ = wallet
                .win(Request::new(WinRequest {
                    user_id: r.user_id.clone(),
                    request_id: r.request_id.clone(),
                    amount: win as i64,
                }))
                .await;
        }

        Ok(Response::new(SpinResponse {
            request_id: r.request_id,
            result,
            win,
        }))
    }
}