# Grafana Plugin SDK for Rust

[![Build Status](https://github.com/grafana/grafana-plugin-sdk-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/grafana/grafana-plugin-sdk-rust/actions/workflows/rust.yml)
[![docs.rs](https://docs.rs/grafana-plugin-sdk/badge.svg)](https://docs.rs/grafana-plugin-sdk)
[![crates.io](https://img.shields.io/crates/v/grafana-plugin-sdk.svg)](https://crates.io/crates/grafana-plugin-sdk)

This is a Rust implementation of the Grafana Plugin SDK for Rust, similar to the [Grafana Plugin SDK for Go][go]. It can be used to build [backend plugins][] for Grafana.

## Current state

This SDK is still in development. The protocol between the Grafana server and the plugin SDK is considered stable, but the convenience functionality in the SDK may experience breaking changes.

**Disclaimer**: this Rust SDK is not (yet) an official Grafana Labs project! Use the [Go SDK][go] if higher maintainability and support standards are required. That being said, this crate will adhere to semantic versioning, and the authors will aim to respond to issues as far as possible.

## Related projects

The [grafana-sample-backend-plugin-rust][sample-plugin] repository contains a sample backend plugin with a backend written in Rust, along with a docker-compose setup with automatic plugin reloading.

## Developing

### Releasing

Releases are handled using [cargo-release][]. Run the following to dry-run release a new version of all crates:

```bash
cargo release <major|minor|patch> --workspace
```

If everything looks OK, add the `--execute` flag to go through with the release.

## License

The Rust SDK is licensed under either of the following, at your option:

- Apache License, Version 2.0, (LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0)
- MIT License (LICENSE-MIT or https://opensource.org/licenses/MIT)

[backend plugins]: https://grafana.com/docs/grafana/latest/developers/plugins/backend/
[cargo-release]: https://crates.io/crates/cargo-release
[go]: https://pkg.go.dev/github.com/grafana/grafana-plugin-sdk-go
[grafana-rs-datasource]: https://github.com/toddtreece/grafana-rs-datasource
[sample-plugin]: https://github.com/sd2k/grafana-sample-backend-plugin-rust/
