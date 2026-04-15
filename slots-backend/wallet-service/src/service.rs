use tonic::{Request, Response, Status};
use proto::wallet::{
    wallet_server::Wallet, BetRequest, BetResponse, WinRequest, WinResponse,
};

use crate::wallet::{deduct_balance, add_balance};

pub struct WalletSvc;

impl Default for WalletSvc {
    fn default() -> Self {
        WalletSvc
    }
}

#[tonic::async_trait]
impl Wallet for WalletSvc {
    async fn bet(
        &self,
        req: Request<BetRequest>,
    ) -> Result<Response<BetResponse>, Status> {
        let r = req.into_inner();

        println!("👉 WalletSvc::bet called");

        match deduct_balance(&r.user_id, r.amount, &r.request_id).await {
            Ok(new_balance) => {
                println!("✅ deduct success, balance={}", new_balance);

                Ok(Response::new(BetResponse {
                    request_id: r.request_id,
                    balance: new_balance,
                    code: 0,
                    msg: "ok".into(),
                }))
            }
            Err(e) => {
                println!("❌ deduct failed: {}", e);

                Ok(Response::new(BetResponse {
                    request_id: r.request_id,
                    balance: 0,
                    code: 1001,
                    msg: e,
                }))
            }
        }
    }

    async fn win(
        &self,
        req: Request<WinRequest>,
    ) -> Result<Response<WinResponse>, Status> {
        let r = req.into_inner();

        println!("👉 WalletSvc::win called");

        match add_balance(&r.user_id, r.amount, &r.request_id).await {
            Ok(new_balance) => {
                println!("✅ add success, balance={}", new_balance);

                Ok(Response::new(WinResponse {
                    request_id: r.request_id,
                    balance: new_balance,
                    code: 0,
                    msg: "ok".into(),
                }))
            }
            Err(e) => {
                println!("❌ add failed: {}", e);

                Ok(Response::new(WinResponse {
                    request_id: r.request_id,
                    balance: 0,
                    code: 1002,
                    msg: e,
                }))
            }
        }
    }
}