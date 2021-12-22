fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.bytes(&[
        ".pluginv2.CallResourceRequest",
        ".pluginv2.CallResourceResponse",
        ".pluginv2.RunStreamRequest",
        ".pluginv2.SubscribeStreamRequest",
    ]);
    Ok(tonic_build::configure().compile_with_config(
        config,
        &["./vendor/proto/backend.proto"],
        &["./vendor/proto"],
    )?)
}
