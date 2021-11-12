# Grafana Plugin SDK for Rust

[![Build Status](https://github.com/sd2k/grafana-plugin-sdk-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/sd2k/grafana-plugin-sdk-rust/actions/workflows/rust.yml)
[![docs.rs](https://docs.rs/grafana-plugin-sdk-rust/badge.svg)](https://docs.rs/grafana-plugin-sdk-rust)
[![crates.io](https://img.shields.io/crates/v/grafana-plugin-sdk-rust.svg)](https://crates.io/crates/grafana-plugin-sdk-rust)

This is a Rust implementation of the Grafana Plugin SDK for Rust, similar to the [Grafana Plugin SDK for Go][go]. It can be used to build [backend plugins][] for Grafana.

## Current state

This SDK is still in development. The protocol between the Grafana server and the plugin SDK is considered stable, but the convenience functionality in the SDK may experience breaking changes.

The Rust SDK is not officially maintained by Grafana Labs; if you need higher support and maintainability guarantees, the [Go SDK][go] is recommended instead. I do intend to keep this SDK up to date with the Go SDK, however!

## Related projects


The [grafana-rs-datasource] also makes use of the Grafana Plugin SDK protobufs and implements a DataFusion datasource in Rust.

[backend plugins]: https://grafana.com/docs/grafana/latest/developers/plugins/backend/
[go]: https://pkg.go.dev/github.com/grafana/grafana-plugin-sdk-go
[grafana-rs-datasource]: https://github.com/toddtreece/grafana-rs-datasource

## License

The Rust SDK is licensed under either of the following, at your option:

- Apache License, Version 2.0, (LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0)
- MIT License (LICENSE-MIT or https://opensource.org/licenses/MIT)
