fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "gen-proto")]
    {
        let mut config = tonic_prost_build::Config::new();
        config.bytes([
            ".pluginv2.CallResourceRequest",
            ".pluginv2.CallResourceResponse",
            ".pluginv2.RunStreamRequest",
            ".pluginv2.SubscribeStreamRequest",
        ]);
        Ok(tonic_prost_build::configure()
            .out_dir("src/pluginv2")
            .compile_with_config(
                config,
                &["./vendor/proto/backend.proto"],
                &["./vendor/proto"],
            )?)
    }
    #[cfg(not(feature = "gen-proto"))]
    Ok(())
}
