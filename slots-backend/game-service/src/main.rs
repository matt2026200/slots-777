use tonic::transport::Server;
use proto::game::game_server::GameServer;

mod service;
mod logic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting GameService on 0.0.0.0:50051...");
    Server::builder()
        .add_service(GameServer::new(service::GameSvc))
        .serve("0.0.0.0:50051".parse()?)
        .await?;

    Ok(())
}
