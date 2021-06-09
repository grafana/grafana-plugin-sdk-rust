Grafana Plugin SDK for Rust
===========================

This is a very WIP experiment at writing a Grafana Plugin SDK for Rust, similar to the [Grafana Plugin SDK for Go][go].

Currently not much is implemented on top of the protobuf definitions, but this could serve as an example of how to implement a backend datasource or app using Rust; see `examples/main.rs` for a basic example.

Related projects
----------------

The [grafana-rs-datasource] also makes use of the Grafana Plugin SDK protobufs and implements a DataFusion datasource in Rust.

[go]: https://pkg.go.dev/github.com/grafana/grafana-plugin-sdk-go.
[grafana-rs-datasource]: https://github.com/toddtreece/grafana-rs-datasource
