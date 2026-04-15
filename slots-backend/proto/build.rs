
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    
    tonic_prost_build::configure()
        .build_server(true) // 生成 server 代码
        .build_client(true) // 生成 client 代码
        .file_descriptor_set_path(format!("{}/descriptor.bin", out_dir)) //生成 descriptor 文件（用于 reflection）
        .compile_protos(
            &[
                "proto/game.proto",
                "proto/wallet.proto", // ✅ 添加 wallet.proto
            ],
            &["proto"], // proto 文件所在目录
        )?;
    Ok(())
}
