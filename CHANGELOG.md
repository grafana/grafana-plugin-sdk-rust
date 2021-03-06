# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Bump arrow2 dependency to 0.12.0
- Bump serde_with dependency to 2.0.0
- Add `headers` field containing the allow-listed fields sent along with the request
  to `CheckHealthRequest` (see [the Go SDK PR](https://github.com/grafana/grafana-plugin-sdk-go/pull/512)
  for more details)
- Add `type_` field containing the plugin type to `DataSourceInstanceSettings`. This is equal
  to the `plugin_id` field on `PluginContext`. See [the Go SDK PR](https://github.com/grafana/grafana-plugin-sdk-go/pull/490)
  for justification.
- Remove unused lifetime on `IntoOptField` blanket impl.

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

[unreleased]: https://github.com/grafana/grafana-plugin-sdk-rust/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/grafana/grafana-plugin-sdk-rust/tag/v0.3.0
[0.2.0]: https://github.com/grafana/grafana-plugin-sdk-rust/tag/v0.2.0
[0.1.0]: https://github.com/grafana/grafana-plugin-sdk-rust/tag/v0.1.0
