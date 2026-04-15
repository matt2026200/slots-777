use tonic::transport::Server;
use tonic_reflection::server::Builder;

use proto::wallet::wallet_server::WalletServer;
use crate::service::WalletSvc;

mod service;
mod wallet;
use proto::FILE_DESCRIPTOR_SET;

// ✅ 这里名字要和 build.rs 里的 descriptor.bin 对应

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50052".parse().unwrap();

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    println!("wallet-service running on 50052");

    Server::builder()
        .add_service(WalletServer::new(WalletSvc::default()))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}