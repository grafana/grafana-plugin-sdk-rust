/*! The Grafana Plugin SDK for Rust.

This crate contains a Rust implementation of the Grafana plugin SDK. It is divided into two main modules:

- [`backend`] contains the traits that must be implemented by backend plugins for various pieces of functionality,
whether querying data, calling resources, or streaming data between Grafana and the plugin.
- [`data`] contains the fundamental data structures used by backend plugins, such as [`Frame`][data::Frame]s, [`Field`][data::Field]s,
and their associated metadata.

Backend plugins communicate with Grafana via gRPC. The low-level protocols are exposed in the [`pluginv2`]
module as an escape hatch, if required. Please file an issue if this is needed and we will try to
accommodate your needs in the next release of the high-level SDK.

See the docs on [backend plugins on grafana.com] for an introduction to backend Grafana plugins.

[Backend plugins on grafana.com]: https://grafana.com/docs/grafana/latest/developers/plugins/backend/
*/
#![cfg_attr(docsrs, feature(doc_notable_trait))]
#![deny(missing_docs)]
#[allow(missing_docs, clippy::all, clippy::nursery, clippy::pedantic)]
pub mod pluginv2 {
    //! The low-level structs generated from protocol definitions.
    tonic::include_proto!("pluginv2");
}

pub mod backend;
pub mod data;

/// Contains useful helper traits for constructing [`Field`]s and [`Frame`]s.
pub mod prelude {
    pub use crate::data::IntoFrame;
    pub use crate::data::{ArrayIntoField, IntoField, IntoOptField};
}
