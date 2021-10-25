fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.bytes(&[
        "CallResourceRequest",
        "CallResourceResponse",
    ]);
    Ok(tonic_build::configure().compile_with_config(
        config,
        &["./vendor/proto/backend.proto"],
        &["./vendor/proto"],
    )?)
}
