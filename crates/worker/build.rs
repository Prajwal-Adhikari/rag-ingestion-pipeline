fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(&["../../proto/splitter.proto"], &["../../proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {}", e));
    Ok(())
}
