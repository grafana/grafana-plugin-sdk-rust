# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- Add `AppInstanceSettings::api_version` and `DataSourceInstanceSettings::api_version` fields.
- Add `PluginContext::grafana_config` field. This can be used to access a new struct,
  `GrafanaConfig`, which contains the configuration passed to the plugin from Grafana.
  Use the methods on `GrafanaConfig` to access the configuration.

### Changed

- Update the vendored protobuf definitions to match version 0.249.0 of the Go SDK.
  This has also added a new field, `api_version`, to the `AppInstanceSettings` and
  `DataSourceInstanceSettings` structs.
- The `ArrayRefIntoField` trait is now correctly hidden behind the `arrow` feature flag.
- Switch from `arrow2` to `arrow`.
  The `arrow2` crate has been deprecated and is no longer maintained. The SDK now uses the
  `arrow` crate instead, which is maintained by the Arrow project.
  This is a breaking change because the `Field::values()` method now returns an
  `Arc<dyn arrow::array::Array>` instead of the `arrow2` equivalent, and the
  APIs for working with arrow arrays differs between the two crates.
  Users who are just using the simple `Field::set_values` method or the various
  `IntoField` / `IntoOptField` traits should not be affected.
- Update MSRV to 1.81
- Bump arrow dependency to 56.0
- Bump darling dependency to 0.21.0
- Bump itertools dependency to 0.14.0
- Bump prometheus dependency to 0.14.0
- Bump thiserror dependency to 2.0.11
- Bump prost, tonic, tonic-build and tonic-health dependencies to 0.14.2; added tonic-prost dependency
- Bump tracing-serde dependency to 0.2.0

## [0.5.0] - 2024-09-17

### Added

- Plugins must now specify a custom type for the `json_data` and
  `decrypted_secure_json_data` fields of their app/datasource instance
  settings, corresponding to the two type parameters of `DataSourceSettings`
  in the `@grafana/data` Javascript library.

  Plugin structs must now also implement the `GrafanaPlugin` trait, which is used
  to declare:
  - the type of the JSON data and secure JSON data
  - the type of plugin (`PluginType::App` or `PluginType::Datasource`)

  The simplest way to do so is to use the `GrafanaPlugin` derive macro exporter from the library's prelude:

  ```rust
  use std::collections::HashMap;

  use grafana_plugin_sdk::prelude::*;
  use serde::DeserializeOwned;

  #[derive(Debug, DeserializeOwned)]
  struct DatasourceSettings {
      max_retries: usize,
      other_custom_setting: String,
  }

  #[derive(Debug, GrafanaPlugin)]
  #[grafana_plugin(
      type = "datasource",
      json_data = "DatasourceSettings",
      secure_json_data = "HashMap<String, String>",
  )]
  struct Plugin {
      // any plugin data
  }
  ```

  Both `json_data` and `secure_json_data` default to `serde_json::Value` if omitted.

  The various `Request` structs in the `backend` module are now type aliases for more complex structs,
  and require a type parameter which should be `Self`:

  ```rust
  impl backend::ResourceService for MyPluginService {

      ...

      async fn call_resource(
          &self,
          r: backend::CallResourceRequest<Self>,  // Note the new `Self` type parameter.
      ) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
          ...
      }
  ```

### Changed

- Bump itertools dependency to 0.13.0
- Bump prost dependency to 0.13.2
- Bump reqwest dependency to 0.12.7
- Bump tonic dependency to 0.12.2
- Bump tonic-health dependency to 0.12.2
- Increase MSRV to 1.63, due to tokio-util requiring it

## [0.4.3] - 2024-01-18

### Added

- Add `VisType::FlameGraph` variant to indicate that a frame should be visualised using the flame graph panel introduced [here](https://github.com/grafana/grafana/pull/56376).
- Add overrideable `DataQueryError::status` method which must return a `DataQueryStatus`. This can be used by datasource implementations to provide more detail about how an error should be handled.
- Add `arrow-array` support for Field ([#111](https://github.com/grafana/grafana-plugin-sdk-rust/pull/111) by @kerryeon)

### Changed

- Change the `plugin_context` field of various structs to be non-optional, matching the Go SDK.
- Plugins will only spawn shutdown handlers when compiled in debug mode. When compiled in release mode the shutdown handler attribute will do nothing.
- Bump arrow2 dependency to 0.18.0
- Bump http dependency to 1.0
- Bump syn dependency to 2.0
- Bump prost dependency to 0.12.3
- Bump tonic dependency to 0.10.2

## [0.4.2] - 2022-09-19

## [0.4.1] - 2022-09-19

## [0.4.0] - 2022-09-19

### Changed

- The `DataService` trait has a new associated type, `Query`, which corresponds to the type of the query sent from the frontend component of the plugin (the `TQuery` type parameter of the frontend `DatasourceApi` implementation). The backend SDK will attempt to deserialize the JSON into this struct, and it will be accessible on the `query` property of each query in `QueryDataRequest.queries`. Note that `QueryDataRequest` is also now generic over this type. Within the `DataService` trait, it is simplest to use `Self::Query` to refer to the new type.
  To retain the old behaviour, set `Query = serde_json::Value` in `DataService`.
- Add `headers` field containing the allow-listed fields sent along with the request
  to `CheckHealthRequest` (see [the Go SDK PR](https://github.com/grafana/grafana-plugin-sdk-go/pull/512)
  for more details)
- Add `type_` field containing the plugin type to `DataSourceInstanceSettings`. This is equal
  to the `plugin_id` field on `PluginContext`. See [the Go SDK PR](https://github.com/grafana/grafana-plugin-sdk-go/pull/490)
  for justification.
- Add impl of `backend::IntoHttpResponse` for `http::Response<Vec<u8>>`.
- Remove unused lifetime on `IntoOptField` blanket impl.
- Derive `Eq` (as well as just `PartialEq`) for various structs across the crate.
- Bump arrow2 dependency to 0.14.0
- Bump prost to 0.11.0 and remove prost-build dependency, preferring checked-in generated code.
  This should speed up build times and remove the dependency on `protoc` since we no longer need to compile proto definitions.
- Bump tonic to 0.8.0 and remove tonic-build dependency.
- Bump serde_with dependency to 2.0.0
- Use cargo-release to automate release process

## [0.3.0] - 2022-04-14

### Added

- Added various new constructors for `SubscribeStreamResponse`, `PublishStreamResponse`
  and `CheckHealthResponse`, reducing the reliance of knowing what the arguments should
  be.

### Deprecated

- The `CheckHealthResponse::new`, `SubscribeStreamResponse::new` and
  `PublishStreamResponse::new` methods have been deprecated in favour of their new,
  more direct constructors.

### Changed

- Dependency bumps:
  - prost 0.9.0 -> 0.10.0
  - tonic 0.6.0 -> 0.7.0
  - tonic-health 0.5.0 -> 0.6.0
- `InitialData::from_json` now only takes the desired JSON `Value` by reference rather than by
  value.

## [0.2.0] - 2022-03-15

### Added

- Bump arrow2 dependency to 0.10.0, and re-export it as `grafana_plugin_sdk::arrow2`
- Added new `data` field to `SubscribeStreamRequest` and `SubscribeStreamResponse`,
  matching the latest release of the protobuf descriptors.
- More types now impl `FieldType` and `IntoFieldType`:
  - `bool`
  - `SystemTime`
  - `chrono::Date`
  - `chrono::NaiveDate`
  - `chrono::NaiveDateTime`
- The `FieldType` and `IntoFieldType` traits are now public. These are useful when
  writing generic functions to convert iterators, vecs, slices or arrays into `Field`s.

### Changed

- Mark the various `Request` and `Response` structs in the backend part of the SDK as
  `#[non_exhaustive]` since changes to those structs are largely outside of our control;
  the protobuf descriptors may add additional fields, and this allows us to include them
  without breaking our API. Some `Response` types now have new constructors which should
  be used.
- Most of the exported types in the `data` part of the SDK are also now non-exhaustive,
  for the same reason as above. These types now either have separate constructors or
  `Default` impls which can be used to create them.
- Derive `Clone` for various backend structs:
  - `AppInstanceSettings`
  - `DataSourceInstanceSettings`
  - `PluginContext`
  - `Role` (is also now `Copy`)
  - `TimeRange`
  - `User`
- Change the `Iter` associated type of `backend::DataService` to `Stream`, and update
  the return type of `query_data` accordingly. This allows each inner query to be handled
  asynchronously and concurrently in a simple way.
- The `live::Error` type is now an enum and provides more detail on failures.
- The `path` field of `SubscribeStreamRequest`, `RunStreamRequest` and
  `PublishStreamRequest` are now `live::Path` types rather than arbitrary strings.

## [0.1.0] - 2021-12-08

### Added

- Initial release of the SDK

<!-- next-url -->
[Unreleased]: https://github.com/assert-rs/predicates-rs/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/assert-rs/predicates-rs/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/assert-rs/predicates-rs/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/assert-rs/predicates-rs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/assert-rs/predicates-rs/compare/v0.4.0...v0.4.1
[unreleased]: https://github.com/grafana/grafana-plugin-sdk-rust/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/grafana/grafana-plugin-sdk-rust/tag/v0.3.0
[0.2.0]: https://github.com/grafana/grafana-plugin-sdk-rust/tag/v0.2.0
[0.1.0]: https://github.com/grafana/grafana-plugin-sdk-rust/tag/v0.1.0
