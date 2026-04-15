pub mod game {
    tonic::include_proto!("game");
}
pub mod wallet {
    tonic::include_proto!("wallet");
}
pub const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("descriptor");