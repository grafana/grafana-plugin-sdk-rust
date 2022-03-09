/*! The Grafana Plugin SDK for Rust.

This crate contains a Rust implementation of the Grafana plugin SDK. It is divided into three main modules:

- [`backend`] contains the traits that must be implemented by backend plugins for various pieces of functionality,
whether querying data, calling resources, or streaming data between Grafana and the plugin.
- [`data`] contains the fundamental data structures used by backend plugins, such as [`Frame`][data::Frame]s, [`Field`][data::Field]s,
and their associated metadata.
- [`live`] contains functionality used by [Grafana Live], the streaming messaging service available from
Grafana 8.0.

The [`prelude`] contains some useful unambiguous traits which are helpful when creating some structures,
particularly [`Frame`][data::Frame]s and [`Field`][data::Field]s.

Backend plugins communicate with Grafana via gRPC. The low-level protocols are exposed in the [`pluginv2`]
module as an escape hatch, if required. Please file an issue if this is needed and we will try to
accommodate your needs in the next release of the high-level SDK.

See the docs on [backend plugins on grafana.com] for an introduction to backend Grafana plugins, or check out
the [crate examples] or [sample app repo] to get started with writing a backend plugin in Rust.

# Feature flags

The following feature flags enable additional functionality for this crate:

- `reqwest` - adds an [`IntoHttpResponse`][crate::backend::IntoHttpResponse] implementation for
    [`reqwest::Response`]

[Backend plugins on grafana.com]: https://grafana.com/docs/grafana/latest/developers/plugins/backend/
[Grafana Live]: https://grafana.com/docs/grafana/latest/live/
[crate examples]: https://github.com/grafana/grafana-plugin-sdk-rust/tree/main/examples
[sample app repo]: https://github.com/sd2k/grafana-sample-backend-plugin-rust/
*/
#![cfg_attr(docsrs, feature(doc_notable_trait))]
#![deny(missing_docs)]
#![feature(generic_associated_types)]

/// Re-export of the arrow2 crate depended on by this crate.
///
/// We recommend that you use this re-export rather than depending on arrow2
/// directly to ensure compatibility; otherwise, rustc/cargo may emit mysterious
/// error messages.
pub use arrow2;

#[cfg(feature = "reqwest")]
extern crate reqwest_lib as reqwest;

#[allow(missing_docs, clippy::all, clippy::nursery, clippy::pedantic)]
pub mod pluginv2 {
    //! The low-level structs generated from protocol definitions.
    tonic::include_proto!("pluginv2");
}

pub mod backend;
pub mod data;
pub mod live;

/// Contains useful helper traits for constructing [`Field`][data::Field]s and [`Frame`][data::Frame]s.
pub mod prelude {
    pub use crate::data::{ArrayIntoField, FromFields, IntoField, IntoFrame, IntoOptField};
}

#[doc(inline)]
pub use grafana_plugin_sdk_macros::*;

/// WARNING: Do not use this method outside of the SDK.
#[doc(hidden)]
pub fn async_main<R>(fut: impl std::future::Future<Output = R> + Send) -> R {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("grafana-plugin-worker-thread")
        .enable_all()
        .build()
        .expect("create tokio runtime")
        .block_on(fut)
}
