/*! Functionality for use by backend plugins.

Backend plugins are executables that expose a gRPC server, which Grafana uses to
communicate all information. This SDK uses [tonic] as its gRPC implementation.

The basic requirements for a backend plugin are to provide a `main` function which:

- first, calls the [`initialize`] function from this module to initialize the plugin. This **must**
  be done before any other code can output to stdout or stderr!
- performs any setup required by the plugin (such as connecting to databases or initializing any state)
- creates the struct representing the plugin's services, which should implement one of the various traits exposed
  by this module
- create a [`Plugin`] to manage the plugin's lifecycle with Grafana
- use the `Plugin::*_service` methods to attach your plugin
- begin serving on the listener returned by `initialize` using [`Plugin::start`].

If you're not sure where to start, take a look at the [`Plugin`] struct, its four important methods, and
the trait bounds for each of them - those give an idea of what your plugin will need to implement.

# Logging and `tracing`

The SDK provides a preconfigured [`tracing_subscriber::fmt::Layer`] which, when installed,
will emit structured logs in a format understood by Grafana. Use the [`layer`] function
to get the `Layer`, and install it using `tracing_subscriber::Registry::with`. Alternatively
use [`Plugin::init_subscriber`] to automatically install a subscriber using the `RUST_LOG`
environment variable to set the directive (defaulting to `info` if not set).

Once the layer is installed, any logs emitted by the various [`log`](https://docs.rs/log) or [`tracing`] macros
will be emitted to Grafana.

# Example

```rust,no_run
use futures_util::stream::FuturesOrdered;
use grafana_plugin_sdk::{backend, data, prelude::*};
use thiserror::Error;
use tonic::transport::Server;
use tracing::info;

#[derive(Clone, Debug)]
struct MyPlugin;

/// An error that may occur during a query.
///
/// This must store the `ref_id` of the query so that Grafana can line it up.
#[derive(Debug, Error)]
#[error("Error querying backend for query {ref_id}: {source}")]
struct QueryError {
    source: data::Error,
    ref_id: String,
}

impl backend::DataQueryError for QueryError {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

#[tonic::async_trait]
impl backend::DataService for MyPlugin {

    /// The type of JSON data sent from Grafana to our backend plugin.
    ///
    /// This will correspond to the `TQuery` type parameter of the frontend
    /// datasource.
    ///
    /// We can use `serde_json::Value` if we want to accept any JSON.
    type Query = serde_json::Value;

    /// The type of error that could be returned by an individual query.
    type QueryError = QueryError;

    /// The type of iterator we're returning.
    ///
    /// In general the concrete type will be impossible to name in advance,
    /// so the `backend::BoxDataResponseStream` type alias will be useful.
    type Stream = backend::BoxDataResponseStream<Self::QueryError>;

    /// Respond to a request for data from Grafana.
    ///
    /// This request will contain zero or more queries, as well as information
    /// about the datasource instance on behalf of which this request is made,
    /// such as address, credentials, etc.
    ///
    /// Our plugin must respond to each query and return an iterator of `DataResponse`s,
    /// which themselves can contain zero or more `Frame`s.
    async fn query_data(&self, request: backend::QueryDataRequest<Self::Query>) -> Self::Stream {
        Box::pin(
            request
                .queries
                .into_iter()
                .map(|x| async {
                    // Here we create a single response Frame for each query.
                    // Frames can be created from iterators of fields using [`IntoFrame`].
                    Ok(backend::DataResponse::new(
                        x.ref_id.clone(),
                        // Return zero or more frames.
                        // A real implementation would fetch this data from a database
                        // or something.
                        vec![[
                            [1_u32, 2, 3].into_field("x"),
                            ["a", "b", "c"].into_field("y"),
                        ]
                        .into_frame("foo")
                        .check()
                        .map_err(|source| QueryError {
                            ref_id: x.ref_id,
                            source,
                        })?],
                    ))
                })
                .collect::<FuturesOrdered<_>>(),
        )
    }
}

#[grafana_plugin_sdk::main(services(data))]
async fn plugin() -> MyPlugin {
    // Create our plugin struct. Any state, such as a database connection, should be
    // held here, perhaps inside an `Arc` if required.
    MyPlugin
}
```

[tonic]: https://github.com/hyperium/tonic
*/
use std::{collections::HashMap, fmt::Debug, io, net::SocketAddr, str::FromStr};

use chrono::prelude::*;
use futures_util::FutureExt;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tracing_subscriber::{
    fmt::{format::JsonFields, time::UtcTime},
    prelude::*,
    registry::LookupSpan,
    EnvFilter,
};

use crate::{
    live,
    pluginv2::{
        self, data_server::DataServer, diagnostics_server::DiagnosticsServer,
        resource_server::ResourceServer, stream_server::StreamServer,
    },
};

/// Re-export of `async_trait` proc macro, so plugin implementations don't have to import tonic manually.
pub use tonic::async_trait;

mod data;
mod diagnostics;
mod noop;
mod resource;
mod stream;
mod tracing_fmt;

pub use data::{
    BoxDataResponseStream, DataQuery, DataQueryError, DataQueryStatus, DataResponse, DataService,
    QueryDataRequest,
};
pub use diagnostics::{
    CheckHealthRequest, CheckHealthResponse, CollectMetricsRequest, CollectMetricsResponse,
    DiagnosticsService, HealthStatus, Payload as MetricsPayload,
};
pub use resource::{
    BoxResourceFuture, BoxResourceStream, CallResourceRequest, ErrIntoHttpResponse,
    IntoHttpResponse, ResourceService,
};
pub use stream::{
    BoxRunStream, InitialData, PublishStreamRequest, PublishStreamResponse, PublishStreamStatus,
    RunStreamRequest, StreamPacket, StreamService, SubscribeStreamRequest, SubscribeStreamResponse,
    SubscribeStreamStatus,
};
pub use tracing_fmt::HCLogJson;

use noop::NoopService;

struct ShutdownHandler {
    address: SocketAddr,
}

impl ShutdownHandler {
    #[must_use]
    fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    fn spawn(self) -> impl std::future::Future<Output = ()> {
        tokio::spawn(async move {
            let listener = TcpListener::bind(&self.address).await.map_err(|e| {
                tracing::warn!("Error creating shutdown handler: {}", e);
                e
            })?;
            tracing::debug!(address = %self.address, "Shutdown handler started on {}", &self.address);
            Ok::<_, std::io::Error>(listener.accept().await.map(|_| ()))
        })
        .map(|_| ())
    }
}

/// Main entrypoint into the Grafana plugin SDK.
///
/// A `Plugin` handles the negotiation with Grafana, adding gRPC health checks,
/// serving the various `Service`s, and gracefully exiting if configured.
///
/// # Shutdown Handler
///
/// A plugin can be configured to gracefully shutdown by calling the [`Plugin::shutdown_handler`]
/// method. This will spawn a new task which simply waits for _any_ TCP connection on
/// the given address, and triggers a graceful shutdown of the underlying gRPC server.
/// This is helpful during development using Docker: a tool such as [`cargo-watch`][cargo-watch]
/// can be used to rebuild on changes, then use `nc` to trigger a shutdown of the plugin. Grafana
/// will automatically plugins when their process exits, so this avoids having to restart the
/// Grafana server on every change. An example configuration for this can be seen in the
/// [grafana-sample-backend-plugin-rust] repository.
///
/// [cargo-watch]: https://crates.io/crates/cargo-watch
/// [grafana-sample-backend-plugin-rust]: https://github.com/sd2k/grafana-sample-backend-plugin-rust
///
/// # Type parameters
///
/// The four type parameters of a `Plugin` represent the services that this plugin
/// is configured to run. When created using [`Plugin::new`] the plugin has the
/// `NoopService` associated with each type, which simply does not register anything
/// for this piece of functionality. Calling the various methods on `Plugin` (such
/// as [`Plugin::data_service`] and [`Plugin::stream_service`]) will return a new
/// `Plugin` with your plugin implementation registered for that functionality.
///
/// The type parameters stand for:
///
/// - `D`, the diagnostic service
/// - `Q`, the data service ('Q' stands for 'query', here)
/// - `R`, the resource service
/// - `S`, the streaming service
///
/// # Example:
///
/// ```rust
/// use futures_util::stream::FuturesOrdered;
/// use grafana_plugin_sdk::{backend, prelude::*};
/// use thiserror::Error;
///
/// #[derive(Clone)]
/// struct MyPlugin;
///
/// /// An error that may occur during a query.
/// ///
/// /// This must store the `ref_id` of the query so that Grafana can line it up.
/// #[derive(Debug, Error)]
/// #[error("Error querying backend for {ref_id}")]
/// struct QueryError {
///     ref_id: String,
/// };
///
/// impl backend::DataQueryError for QueryError {
///     fn ref_id(self) -> String {
///         self.ref_id
///     }
/// }
///
/// #[tonic::async_trait]
/// impl backend::DataService for MyPlugin {
///
///     /// The type of JSON data sent from Grafana to our backend plugin.
///     ///
///     /// This will correspond to the `TQuery` type parameter of the frontend
///     /// datasource.
///     ///
///     /// We can use `serde_json::Value` if we want to accept any JSON.
///     type Query = serde_json::Value;
///
///     /// The type of error that could be returned by an individual query.
///     type QueryError = QueryError;
///
///     /// The type of iterator we're returning.
///     ///
///     /// In general the concrete type will be impossible to name in advance,
///     /// so the `backend::BoxDataResponseStream` type alias will be useful.
///     type Stream = backend::BoxDataResponseStream<Self::QueryError>;
///
///     /// Respond to a request for data from Grafana.
///     ///
///     /// This request will contain zero or more queries, as well as information
///     /// about the datasource instance on behalf of which this request is made,
///     /// such as address, credentials, etc.
///     ///
///     /// Our plugin must respond to each query and return an iterator of `DataResponse`s,
///     /// which themselves can contain zero or more `Frame`s.
///     async fn query_data(&self, request: backend::QueryDataRequest<Self::Query>) -> Self::Stream {
///         Box::pin(
///             request
///                 .queries
///                 .into_iter()
///                 .map(|x| async {
///                     // Here we create a single response Frame for each query.
///                     // Frames can be created from iterators of fields using [`IntoFrame`].
///                     Ok(backend::DataResponse::new(
///                         x.ref_id.clone(),
///                         // Return zero or more frames.
///                         // A real implementation would fetch this data from a database
///                         // or something.
///                         vec![[
///                             [1_u32, 2, 3].into_field("x"),
///                             ["a", "b", "c"].into_field("y"),
///                         ]
///                         .into_frame("foo")
///                         .check()
///                         .map_err(|source| QueryError {
///                             ref_id: x.ref_id,
///                         })?],
///                     ))
///                 })
///                 .collect::<FuturesOrdered<_>>(),
///         )
///     }
/// }
///
/// #[grafana_plugin_sdk::main(
///     services(data),
///     shutdown_handler = "0.0.0.0:10001",
/// )]
/// async fn plugin() -> MyPlugin {
///     MyPlugin
/// }
/// ```
pub struct Plugin<D, Q, R, S> {
    shutdown_handler: Option<ShutdownHandler>,
    init_subscriber: bool,

    diagnostics_service: Option<D>,
    data_service: Option<Q>,
    resource_service: Option<R>,
    stream_service: Option<S>,
}

impl Plugin<NoopService, NoopService, NoopService, NoopService> {
    /// Create a new `Plugin` with no registered services.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shutdown_handler: None,
            init_subscriber: false,
            diagnostics_service: None,
            data_service: None,
            resource_service: None,
            stream_service: None,
        }
    }
}

impl Default for Plugin<NoopService, NoopService, NoopService, NoopService> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D, Q, R, S> Plugin<D, Q, R, S> {
    /// Add a shutdown handler to the plugin, listening on the specified address.
    ///
    /// The shutdown handler waits for a TCP connection on the specified address
    /// and requests that the server gracefully shutdown when any connection is made.
    ///
    /// Note that this _only_ takes effect in debug mode. The shutdown handler
    /// is never added when the crate is compiled in release mode.
    #[must_use]
    pub fn shutdown_handler(mut self, address: SocketAddr) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                self.shutdown_handler = Some(ShutdownHandler::new(address));
            } else {
                self.shutdown_handler = None;
            }
        }
        self
    }

    /// Initialize a default [`tracing_subscriber::fmt::Subscriber`] upon starting.
    ///
    /// If enabled, this will initialize a `Subscriber` emitting logs to stderr
    /// in a format compatible with Grafana, at a default max level of `info`.
    ///
    /// This effectively causes the following to be called as part of `Plugin::start`:
    ///
    /// ```rust
    /// use grafana_plugin_sdk::backend;
    /// use tracing_subscriber::{prelude::*, EnvFilter};
    ///
    /// let filter = EnvFilter::try_from_default_env()
    ///     .unwrap_or_else(|_| EnvFilter::new("info"));
    /// tracing_subscriber::registry()
    ///     .with(filter)
    ///     .with(backend::layer())
    ///     .init()
    /// ```
    #[must_use]
    pub fn init_subscriber(mut self, init_subscriber: bool) -> Self {
        self.init_subscriber = init_subscriber;
        self
    }

    /// Add a data service to this plugin.
    pub fn data_service<T>(self, service: T) -> Plugin<D, T, R, S>
    where
        T: DataService + Send + Sync + 'static,
    {
        Plugin {
            data_service: Some(service),
            shutdown_handler: self.shutdown_handler,
            init_subscriber: self.init_subscriber,
            diagnostics_service: self.diagnostics_service,
            resource_service: self.resource_service,
            stream_service: self.stream_service,
        }
    }

    /// Add a diagnostics service to this plugin.
    pub fn diagnostics_service<T>(self, service: T) -> Plugin<T, Q, R, S>
    where
        T: DiagnosticsService + Send + Sync + 'static,
    {
        Plugin {
            diagnostics_service: Some(service),
            shutdown_handler: self.shutdown_handler,
            init_subscriber: self.init_subscriber,
            data_service: self.data_service,
            resource_service: self.resource_service,
            stream_service: self.stream_service,
        }
    }

    /// Add a resource service to this plugin.
    pub fn resource_service<T>(self, service: T) -> Plugin<D, Q, T, S>
    where
        T: ResourceService + Send + Sync + 'static,
    {
        Plugin {
            resource_service: Some(service),
            shutdown_handler: self.shutdown_handler,
            init_subscriber: self.init_subscriber,
            diagnostics_service: self.diagnostics_service,
            data_service: self.data_service,
            stream_service: self.stream_service,
        }
    }

    /// Add a streaming service to this plugin.
    pub fn stream_service<T>(self, service: T) -> Plugin<D, Q, R, T>
    where
        T: StreamService + Send + Sync + 'static,
    {
        Plugin {
            stream_service: Some(service),
            shutdown_handler: self.shutdown_handler,
            init_subscriber: self.init_subscriber,
            diagnostics_service: self.diagnostics_service,
            data_service: self.data_service,
            resource_service: self.resource_service,
        }
    }
}

impl<D, Q, R, S> Plugin<D, Q, R, S>
where
    D: DiagnosticsService + Send + Sync + 'static,
    Q: DataService + Send + Sync + 'static,
    R: ResourceService + Send + Sync + 'static,
    S: StreamService + Send + Sync + 'static,
{
    /// Start the plugin.
    ///
    /// This adds all of the configured services, spawns a shutdown handler
    /// (if configured), and blocks while the plugin runs.
    ///
    /// # Panics
    ///
    /// This will panic if `init_subscriber(true)` has been set and another
    /// global subscriber has already been installed. If you are initializing your
    /// own `Subscriber`, you should instead use the [`layer`] function to add a
    /// Grafana-compatible `tokio_subscriber::fmt::Layer` to your subscriber.
    pub async fn start(self, listener: TcpListener) -> Result<(), Error> {
        if self.init_subscriber {
            let filter =
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
            tracing_subscriber::registry()
                .with(layer())
                .with(filter)
                .init();
        }
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        let mut plugins = Vec::with_capacity(5);
        if self.diagnostics_service.is_some() {
            health_reporter.set_serving::<DiagnosticsServer<D>>().await;
            plugins.push("diagnostics");
        }
        if self.data_service.is_some() {
            health_reporter.set_serving::<DataServer<Q>>().await;
            plugins.push("data");
        }
        if self.resource_service.is_some() {
            health_reporter.set_serving::<ResourceServer<R>>().await;
            plugins.push("resource");
        }
        if self.stream_service.is_some() {
            health_reporter.set_serving::<StreamServer<S>>().await;
            plugins.push("stream");
        }
        // Log the services included in the plugin, identically to the Go SDK.
        tracing::debug!(
            plugins = %format!("[{}]", plugins.join(" ")),
            "Serving plugin"
        );
        let router = tonic::transport::Server::builder()
            .trace_fn(|_| tracing::debug_span!("grafana-plugin-sdk"))
            .add_service(health_service)
            .add_optional_service(self.diagnostics_service.map(DiagnosticsServer::new))
            .add_optional_service(self.data_service.map(DataServer::new))
            .add_optional_service(self.resource_service.map(ResourceServer::new))
            .add_optional_service(self.stream_service.map(StreamServer::new));
        if let Some(handler) = self.shutdown_handler {
            let handler = handler.spawn();
            router
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), handler.map(|_| ()))
                .await?;
        } else {
            router
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await?;
        }
        Ok(())
    }
}

/// Initialize the plugin, returning the [`TcpListener`] that the gRPC service should serve on.
///
/// The compiled plugin executable is run by Grafana's backend and is expected
/// to behave as a [go-plugin]. This function initializes the plugin by binding to
/// an available IPv4 address and printing the address and protocol to stdout,
/// which the [go-plugin] infrastructure requires.
///
/// See [the guide on non-Go languages][guide] more details.
///
/// [go-plugin]: https://github.com/hashicorp/go-plugin
/// [guide]: https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md
pub async fn initialize() -> Result<TcpListener, io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    println!("1|2|tcp|{}|grpc", listener.local_addr()?);
    Ok(listener)
}

const HCLOG_TIME_FORMAT: &[time::format_description::FormatItem] = time::macros::format_description!(
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]+00:00"
);

/// Create a `tracing` [`Layer`][tracing_subscriber::Layer] configured to log events in a format understood by Grafana.
///
/// The returned layer should be installed into the tracing subscriber registry, with an optional env filter.
///
/// # Example
///
/// Installing the layer with the default `EnvFilter` (using the `RUST_LOG` environment variable):
///
/// ```rust
/// use grafana_plugin_sdk::backend;
/// use tracing_subscriber::{prelude::*, EnvFilter};
///
/// tracing_subscriber::registry()
///     .with(backend::layer())
///     .with(EnvFilter::from_default_env())
///     .init();
/// ```
pub fn layer<S: tracing::Subscriber + for<'a> LookupSpan<'a>>(
) -> tracing_subscriber::fmt::Layer<S, JsonFields, tracing_fmt::HCLogJson, fn() -> io::Stderr> {
    tracing_subscriber::fmt::layer()
        .with_timer(UtcTime::new(HCLOG_TIME_FORMAT))
        .with_writer(io::stderr as fn() -> std::io::Stderr)
        .event_format(tracing_fmt::HCLogJson::default())
        .fmt_fields(JsonFields::new())
}

/// Errors returned by plugin backends.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An error occurred converting data from Grafana.
    #[error("error converting from Grafana: {0}")]
    ConvertFrom(#[from] ConvertFromError),
    /// An error occurred converting data to Grafana.
    #[error("error converting to Grafana: {0}")]
    ConvertTo(#[from] ConvertToError),
    /// An error occurred while starting the plugin.
    #[error("error serving plugin: {0}")]
    Serve(#[from] tonic::transport::Error),
}

/// Errors occurring when trying to interpret data passed from Grafana to this SDK.
///
/// Generally any errors should be considered a bug and should be reported.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConvertFromError {
    /// The `time_range` was missing from the query.
    #[error("time_range missing from query")]
    MissingTimeRange,
    /// The timestamp passed from Grafana was invalid.
    #[error("error interpreting timestamp ({timestamp}) from Grafana")]
    InvalidTimestamp {
        /// The original timestamp in milliseconds.
        timestamp: i64,
    },
    /// The `plugin_context` was missing from the request.
    #[error("plugin_context missing from request")]
    MissingPluginContext,
    /// The JSON provided by Grafana was invalid.
    #[error("invalid JSON (got {json}): {err}")]
    InvalidJson {
        /// The underlying JSON error.
        err: serde_json::Error,
        /// The JSON for which deserialization was attempted.
        json: String,
    },
    /// The frame provided by Grafana was malformed.
    #[error("invalid frame: {source}")]
    InvalidFrame {
        /// The underlying JSON error.
        source: serde_json::Error,
    },
    /// The resource request was not a valid HTTP request.
    #[error("invalid HTTP request: {source}")]
    InvalidRequest {
        /// The underlying `http` error.
        source: http::Error,
    },
    /// The role string provided by Grafana didn't match the roles known by the SDK.
    #[error("Unknown role: {0}")]
    UnknownRole(String),
    /// The path provided by Grafana was invalid.
    #[error("Invalid path: {source}")]
    InvalidPath {
        /// The underlying reason for the error.
        #[from]
        source: live::ChannelError,
    },
}

impl From<ConvertFromError> for tonic::Status {
    fn from(other: ConvertFromError) -> Self {
        Self::invalid_argument(other.to_string())
    }
}

impl ConvertFromError {
    fn into_tonic_status(self) -> tonic::Status {
        self.into()
    }
}

/// Errors occurring when trying to convert data into a format understood by Grafana.
///
/// If these errors occur it probably means something invalid was passed to the SDK
/// somehow, such as a `HashMap` with numeric keys passed as a JSON value.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConvertToError {
    /// The resource response was not a valid HTTP response.
    #[error("invalid HTTP response")]
    InvalidResponse,
    /// The JSON provided was invalid.
    #[error("invalid JSON: {err}")]
    InvalidJson {
        /// The underlying JSON error.
        err: serde_json::Error,
    },
    /// The frame provided could not be serialized.
    #[error("invalid frame: {source}")]
    InvalidFrame {
        /// The underlying JSON error.
        source: serde_json::Error,
    },
}

type ConvertFromResult<T> = std::result::Result<T, ConvertFromError>;

fn read_json<T>(jdoc: &[u8]) -> ConvertFromResult<T>
where
    T: DeserializeOwned,
{
    // Grafana sometimes sends an empty string instead of an empty map, probably
    // because of some zero value Golang stuff?
    let jdoc = jdoc.is_empty().then(|| b"{}".as_slice()).unwrap_or(jdoc);
    serde_json::from_slice(jdoc).map_err(|err| ConvertFromError::InvalidJson {
        err,
        json: String::from_utf8(jdoc.to_vec())
            .unwrap_or_else(|_| format!("non-utf8 string: {}", String::from_utf8_lossy(jdoc))),
    })
}

/// The time range for a query.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct TimeRange {
    /// The start time of the query.
    pub from: DateTime<Utc>,
    /// The end time of the query.
    pub to: DateTime<Utc>,
}

impl From<pluginv2::TimeRange> for TimeRange {
    fn from(other: pluginv2::TimeRange) -> Self {
        Self {
            from: Utc
                .timestamp_millis_opt(other.from_epoch_ms)
                .single()
                .expect("'from' timestamp is invalid"),
            to: Utc
                .timestamp_millis_opt(other.to_epoch_ms)
                .single()
                .expect("'to' timestamp is invalid"),
        }
    }
}

/// A role within Grafana.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Role {
    /// Admin users can perform any administrative action, such as adding and removing users and datasources.
    Admin,
    /// Editors can create, modify and delete dashboards, and can access the Explore page, but cannot modify users or permissions.
    Editor,
    /// Viewers can view dashboards, but cannot edit them or access the Explore page.
    Viewer,
}

impl FromStr for Role {
    type Err = ConvertFromError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Admin" => Self::Admin,
            "Editor" => Self::Editor,
            "Viewer" => Self::Viewer,
            _ => return Err(ConvertFromError::UnknownRole(s.to_string())),
        })
    }
}

/// A Grafana user.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct User {
    /// The user's login.
    pub login: String,
    /// The user's display name.
    pub name: String,
    /// The user's email.
    pub email: String,
    /// The user's role.
    pub role: Role,
}

impl TryFrom<pluginv2::User> for User {
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::User) -> Result<Self, Self::Error> {
        Ok(Self {
            login: other.login,
            name: other.name,
            email: other.email,
            role: other.role.parse()?,
        })
    }
}

/// Settings for an app instance.
///
/// An app instance is an app plugin of a certain type that has been configured
/// and enabled in a Grafana organisation.
#[derive(Clone)]
#[non_exhaustive]
pub struct AppInstanceSettings {
    /// Includes the non-secret settings of the app instance (excluding datasource config).
    pub json_data: Value,
    /// Key-value pairs where the encrypted configuration in Grafana server have been
    /// decrypted before passing them to the plugin.
    ///
    /// This data is not accessible to the Grafana frontend after it has been set, and should
    /// be used for any secrets (such as API keys or passwords).
    pub decrypted_secure_json_data: HashMap<String, String>,
    /// The last time the configuration for the app plugin instance was updated.
    pub updated: DateTime<Utc>,
}

impl Debug for AppInstanceSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppInstanceSettings")
            .field("json_data", &self.json_data)
            .field("decrypted_secure_json_data", &"<redacted>")
            .field("updated", &self.updated)
            .finish()
    }
}

impl TryFrom<pluginv2::AppInstanceSettings> for AppInstanceSettings {
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::AppInstanceSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            decrypted_secure_json_data: other.decrypted_secure_json_data,
            json_data: read_json(&other.json_data)?,
            updated: Utc
                .timestamp_millis_opt(other.last_updated_ms)
                .single()
                .ok_or(ConvertFromError::InvalidTimestamp {
                    timestamp: other.last_updated_ms,
                })?,
        })
    }
}

/// Settings for a datasource instance.
///
/// A datasource is a datasource plugin of a certain type that has been configured
/// and created in a Grafana organisation. For example, the 'datasource' may be
/// the Prometheus datasource plugin, and there may be many configured Prometheus
/// datasource instances configured in a Grafana organisation.
#[derive(Clone)]
#[non_exhaustive]
pub struct DataSourceInstanceSettings {
    /// The Grafana assigned numeric identifier of the the datasource instance.
    pub id: i64,

    /// The Grafana assigned string identifier of the the datasource instance.
    pub uid: String,

    /// The unique identifier of the plugin that the request is for.
    ///
    /// This should be the same value as `[PluginContext.plugin_id]`.
    pub type_: String,

    /// The configured name of the datasource instance.
    pub name: String,

    /// The configured URL of a datasource instance (e.g. the URL of an API endpoint).
    pub url: String,

    /// A configured user for a datasource instance. This is not a Grafana user, rather an arbitrary string.
    pub user: String,

    /// The configured database for a datasource instance. (e.g. the default Database a SQL datasource would connect to).
    pub database: String,

    /// Indicates if this datasource instance should use basic authentication.
    pub basic_auth_enabled: bool,

    /// The configured user for basic authentication.
    ///
    /// E.g. when a datasource uses basic authentication to connect to whatever API it fetches data from.
    pub basic_auth_user: String,

    /// The raw DataSourceConfig as JSON as stored by the Grafana server.
    ///
    /// It repeats the properties in this object and includes custom properties.
    pub json_data: Value,

    /// Key-value pairs where the encrypted configuration in Grafana server have been
    /// decrypted before passing them to the plugin.
    ///
    /// This data is not accessible to the Grafana frontend after it has been set, and should
    /// be used for any secrets (such as API keys or passwords).
    pub decrypted_secure_json_data: HashMap<String, String>,

    /// The last time the configuration for the datasource instance was updated.
    pub updated: DateTime<Utc>,
}

impl Debug for DataSourceInstanceSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataSourceInstanceSettings")
            .field("id", &self.id)
            .field("type_", &self.type_)
            .field("uid", &self.uid)
            .field("name", &self.name)
            .field("url", &self.url)
            .field("user", &self.user)
            .field("database", &self.database)
            .field("basic_auth_enabled", &self.basic_auth_enabled)
            .field("basic_auth_user", &self.basic_auth_user)
            .field("json_data", &self.json_data)
            .field("decrypted_secure_json_data", &"<redacted>")
            .field("updated", &self.updated)
            .finish()
    }
}

impl TryFrom<(pluginv2::DataSourceInstanceSettings, String)> for DataSourceInstanceSettings {
    type Error = ConvertFromError;
    fn try_from(
        (other, type_): (pluginv2::DataSourceInstanceSettings, String),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: other.id,
            uid: other.uid,
            type_,
            name: other.name,
            url: other.url,
            user: other.user,
            database: other.database,
            basic_auth_enabled: other.basic_auth_enabled,
            basic_auth_user: other.basic_auth_user,
            decrypted_secure_json_data: other.decrypted_secure_json_data,
            json_data: read_json(&other.json_data)?,
            updated: Utc
                .timestamp_millis_opt(other.last_updated_ms)
                .single()
                .ok_or(ConvertFromError::InvalidTimestamp {
                    timestamp: other.last_updated_ms,
                })?,
        })
    }
}

/// Holds contextual information about a plugin request: Grafana org, user, and plugin instance settings.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PluginContext {
    /// The organisation ID from which the request originated.
    pub org_id: i64,

    /// The ID of the plugin.
    pub plugin_id: String,

    /// Details about the Grafana user who made the request.
    ///
    /// This will be `None` if the Grafana backend initiated the request,
    /// such as when the request is made on behalf of Grafana Alerting.
    pub user: Option<User>,

    /// The configured app instance settings.
    ///
    /// An app instance is an app plugin of a certain type that has been configured
    /// and enabled in a Grafana organisation.
    ///
    /// This will be `None` if the request does not target an app instance.
    pub app_instance_settings: Option<AppInstanceSettings>,

    /// The configured datasource instance settings.
    ///
    /// A datasource instance is a datasource plugin of a certain type that has been configured
    /// and created in a Grafana organisation. For example, the 'datasource' may be
    /// the Prometheus datasource plugin, and there may be many configured Prometheus
    /// datasource instances configured in a Grafana organisation.
    ///
    /// This will be `None` if the request does not target a datasource instance.
    pub datasource_instance_settings: Option<DataSourceInstanceSettings>,
}

impl TryFrom<pluginv2::PluginContext> for PluginContext {
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::PluginContext) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: other.org_id,
            plugin_id: other.plugin_id.clone(),
            user: other.user.map(TryInto::try_into).transpose()?,
            app_instance_settings: other
                .app_instance_settings
                .map(TryInto::try_into)
                .transpose()?,
            datasource_instance_settings: other
                .data_source_instance_settings
                .map(|ds| (ds, other.plugin_id).try_into())
                .transpose()?,
        })
    }
}
