fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 编译 supervisor.proto 和 keyserver.proto
    // keyserver.proto 导入了 supervisor.proto，所以需要在 includes 中包含两个目录
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "../supervit/proto/supervisor.proto",
                "proto/keyserver.proto",
            ],
            &[
                "../supervit/proto", // 用于查找 supervisor.proto
                "proto",             // 用于查找 keyserver.proto
            ],
        )?;

    // 告诉 cargo 如果 proto 文件变化则重新构建
    println!("cargo:rerun-if-changed=proto/keyserver.proto");
    println!("cargo:rerun-if-changed=../supervit/proto/supervisor.proto");

    Ok(())
}
