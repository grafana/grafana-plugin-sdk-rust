Grafana Plugin SDK for Rust
===========================

This is a Rust implementation of the Grafana Plugin SDK for Rust, similar to the [Grafana Plugin SDK for Go][go].

## Status

Many of the pieces are now at least partially implemented, including useful idiomatic wrappers around the main datatypes (`data::Frame`s and `data::Field`s) and services (`backend::DataService` and `backend::StreamService`), which handle serialization of data before it is sent back to Grafana.

See `examples/main.rs` for an example of how to write a basic datasource + streaming plugin.

Related projects
----------------

The [grafana-rs-datasource] also makes use of the Grafana Plugin SDK protobufs and implements a DataFusion datasource in Rust.

[go]: https://pkg.go.dev/github.com/grafana/grafana-plugin-sdk-go.
[grafana-rs-datasource]: https://github.com/toddtreece/grafana-rs-datasource
