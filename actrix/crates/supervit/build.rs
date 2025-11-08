fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/supervisor.proto")?;

    // 告诉 cargo 如果 proto 文件变化则重新构建
    println!("cargo:rerun-if-changed=proto/supervisor.proto");

    Ok(())
}
