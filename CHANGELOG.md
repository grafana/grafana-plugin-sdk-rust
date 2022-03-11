# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Bump arrow2 dependency to 0.9.0, and re-export it as `grafana_plugin_sdk::arrow2`
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

## [0.1.0] - 2021-12-08

### Added

- Initial release of the SDK

[unreleased]: https://github.com/sd2k/grafana-plugin-sdk-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sd2k/grafana-plugin-sdk-rust/tag/v0.1.0
