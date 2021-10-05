/*! Functionality for use by backend plugins.

Backend plugins are executables that expose a gRPC server, which Grafana uses to
communicate all information. This SDK uses [tonic] as its gRPC implementation.

The basic requirements for a plugin are to provide a `main` function which:

- first, calls the [`initialize`] function from this module to initialize the plugin
- performs any setup required by the plugin (such as connecting to databases or initializing any state)
- creates the struct representing the plugin, which should implement one of the various traits exposed
  by this module
- create 'services' by wrapping the plugin struct in the various `Server` structs re-exported from the `pluginv2` module
- create a [`tonic::transport::Server`] and add the services using `add_service`
- begin serving on the address returned by `initialize`.

# Example

```rust,no_run
use grafana_plugin_sdk::{backend, prelude::*};
use thiserror::Error;
use tonic::transport::Server;

struct Plugin;

/// An error that may occur during a query.
///
/// This must store the `ref_id` of the query so that Grafana can line it up.
#[derive(Debug, Error)]
#[error("Error querying backend for {ref_id}")]
struct QueryError {
    ref_id: String,
};

impl backend::DataQueryError for QueryError {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

#[tonic::async_trait]
impl backend::DataService for Plugin {
    type QueryError = QueryError;
    type Iter = backend::BoxDataResponseIter<Self::QueryError>;
    async fn query_data(&self, request: backend::QueryDataRequest) -> Self::Iter {
        Box::new(
            request.queries.into_iter().map(|x| {
                Ok(backend::DataResponse::new(
                    // Include the ID of the query in the response.
                    x.ref_id,
                    // Return one or more frames.
                    // A real implementation would fetch this data from a database
                    // or something.
                    vec![
                        [
                            [1_u32, 2, 3].into_field("x"),
                            ["a", "b", "c"].into_field("y"),
                        ]
                        .into_frame("foo"),
                    ],
                ))
            })
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = backend::initialize().await?;

    // Create our plugin struct. Any state, such as a database connection, should be
    // held here, perhaps inside an `Arc` if required.
    let plugin = Plugin;

    // Our plugin implements `backend::DataService`, so we can wrap it like so.
    let data_svc = backend::DataServer::new(plugin);

    // Serve forever.
    Server::builder().add_service(data_svc).serve(addr).await?;
    Ok(())
}
```

[tonic]: https://github.com/hyperium/tonic.
*/
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    io,
    net::SocketAddr,
    str::FromStr,
};

use chrono::prelude::*;
use serde_json::Value;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::pluginv2;

/// Re-export of `async_trait` proc macro, so plugin implementations don't have to import tonic manually.
pub use tonic::async_trait;

mod data;
mod diagnostics;
mod resource;
mod stream;

pub use data::{
    BoxDataResponseIter, DataQuery, DataQueryError, DataResponse, DataService, QueryDataRequest,
};
pub use diagnostics::{
    CheckHealthRequest, CheckHealthResponse, CollectMetricsRequest, CollectMetricsResponse,
    DiagnosticsService, HealthStatus,
};
pub use pluginv2::{
    data_server::DataServer, diagnostics_server::DiagnosticsServer,
    resource_server::ResourceServer, stream_server::StreamServer,
};
pub use resource::{BoxResourceStream, CallResourceRequest, ResourceService};
pub use stream::{
    BoxRunStream, InitialData, PublishStreamRequest, PublishStreamResponse, RunStreamRequest,
    StreamPacket, StreamService, SubscribeStreamRequest, SubscribeStreamResponse,
    SubscribeStreamStatus,
};

/// Initialize the plugin, returning the address that the gRPC service should serve on.
///
/// The compiled plugin executable is run by Grafana's backend and is expected
/// to behave as a [go-plugin]. This function initializes the plugin by binding to
/// an available IPv6 address and printing the address and protocol to stdout,
/// which the [go-plugin] infrastructure requires.
///
/// See [the guide on non-Go languages][guide] more details.
///
/// [go-plugin]: https://github.com/hashicorp/go-plugin
/// [guide]: https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md
pub async fn initialize() -> Result<SocketAddr, io::Error> {
    let address = TcpListener::bind("127.0.0.1:0").await?.local_addr()?;
    println!("1|2|tcp|{}|grpc", address);
    Ok(address)
}

/// Errors returned by plugin backends.
///
/// This is very subject to change and should not be relied upon.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An error occurred converting data to/from protobufs.
    #[error("error converting to or from proto: {0}")]
    Conversion(#[from] ConversionError),
    /// An error occurred while starting the plugin.
    #[error("error starting plugin: {0}")]
    Start(#[from] io::Error),
    /// An error occurred while starting the plugin.
    #[error("error serving plugin: {0}")]
    Serve(#[from] tonic::transport::Error),
}

/// Errors occurring when trying to interpret data passed from Grafana to this SDK.
///
/// Generally any errors should be considered a bug and should be reported.
///
/// This is very subject to change and should not be relied upon.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConversionError {
    /// The `time_range` was missing from the query.
    #[error("time_range missing from query")]
    MissingTimeRange,
    /// The `plugin_context` was missing from the request.
    #[error("plugin_context missing from request")]
    MissingPluginContext,
    /// The JSON provided by Grafana was invalid.
    #[error("invalid JSON (got {json}): {err}")]
    InvalidJson {
        /// The underlying JSON error.
        err: serde_json::Error,
        /// The JSON for which (de)serialization was attempted.
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
    /// The resource response was not a valid HTTP response.
    #[error("invalid HTTP response")]
    InvalidResponse,
    /// The role string provided by Grafana didn't match the roles known by the SDK.
    #[error("Unknown role: {0}")]
    UnknownRole(String),
}

impl From<ConversionError> for tonic::Status {
    fn from(other: ConversionError) -> Self {
        Self::invalid_argument(other.to_string())
    }
}

impl ConversionError {
    fn into_tonic_status(self) -> tonic::Status {
        self.into()
    }
}

type ConversionResult<T> = std::result::Result<T, ConversionError>;

pub(self) fn read_json(jdoc: &[u8]) -> ConversionResult<Value> {
    // Grafana sometimes sends an empty string instead of an empty map, probably
    // because of some zero value Golang stuff?
    (!jdoc.is_empty())
        .then(|| {
            serde_json::from_slice(jdoc).map_err(|err| ConversionError::InvalidJson {
                err,
                json: String::from_utf8(jdoc.to_vec()).unwrap_or_else(|_| {
                    format!("non-utf8 string: {}", String::from_utf8_lossy(jdoc))
                }),
            })
        })
        .unwrap_or_else(|| Ok(serde_json::json!({})))
}

/// The time range for a query.
#[derive(Debug)]
pub struct TimeRange {
    /// The start time of the query.
    pub from: DateTime<Utc>,
    /// The end time of the query.
    pub to: DateTime<Utc>,
}

impl From<pluginv2::TimeRange> for TimeRange {
    fn from(other: pluginv2::TimeRange) -> Self {
        Self {
            from: Utc.timestamp_millis(other.from_epoch_ms),
            to: Utc.timestamp_millis(other.to_epoch_ms),
        }
    }
}

/// A role within Grafana.
#[derive(Debug)]
pub enum Role {
    /// Admin users can perform any administrative action, such as adding and removing users and datasources.
    Admin,
    /// Editors can create, modify and delete dashboards, and can access the Explore page, but cannot modify users or permissions.
    Editor,
    /// Viewers can view dashboards, but cannot edit them or access the Explore page.
    Viewer,
}

impl FromStr for Role {
    type Err = ConversionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Admin" => Self::Admin,
            "Editor" => Self::Editor,
            "Viewer" => Self::Viewer,
            _ => return Err(ConversionError::UnknownRole(s.to_string())),
        })
    }
}

/// A Grafana user.
#[derive(Debug)]
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
    type Error = ConversionError;
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
#[derive(Debug)]
pub struct AppInstanceSettings {
    /// Includes the non-secret settings of the app instance (excluding datasource config).
    pub json_data: Value,
    /// Key-value pairs where the encrypted configuration in Grafana server have been
    /// decrypted before passing them to the plugin.
    ///
    /// This data is not accessible to the Grafana frontend after it has been set, and should
    /// be used for any secrets (such as API keys or passwords).
    pub decrypted_secure_json_data: HashMap<String, String>,
    /// The last time the configuratino for the app plugin instance was updated.
    pub updated: DateTime<Utc>,
}

impl TryFrom<pluginv2::AppInstanceSettings> for AppInstanceSettings {
    type Error = ConversionError;
    fn try_from(other: pluginv2::AppInstanceSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            decrypted_secure_json_data: other.decrypted_secure_json_data,
            json_data: read_json(&other.json_data)?,
            updated: Utc.timestamp_millis(other.last_updated_ms),
        })
    }
}

/// Settings for a datasource instance.
///
/// A datasource is a datasource plugin of a certain type that has been configured
/// and created in a Grafana organisation. For example, the 'datasource' may be
/// the Prometheus datasource plugin, and there may be many configured Prometheus
/// datasource instances configured in a Grafana organisation.
#[derive(Debug)]
pub struct DataSourceInstanceSettings {
    /// The Grafana assigned numeric identifier of the the datasource instance.
    pub id: i64,

    /// The Grafana assigned string identifier of the the datasource instance.
    pub uid: String,

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

impl TryFrom<pluginv2::DataSourceInstanceSettings> for DataSourceInstanceSettings {
    type Error = ConversionError;
    fn try_from(other: pluginv2::DataSourceInstanceSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            id: other.id,
            uid: other.uid,
            name: other.name,
            url: other.url,
            user: other.user,
            database: other.database,
            basic_auth_enabled: other.basic_auth_enabled,
            basic_auth_user: other.basic_auth_user,
            decrypted_secure_json_data: other.decrypted_secure_json_data,
            json_data: read_json(&other.json_data)?,
            updated: Utc.timestamp_millis(other.last_updated_ms),
        })
    }
}

/// Holds contextual information about a plugin request: Grafana org, user, and plugin instance settings.
#[derive(Debug)]
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
    type Error = ConversionError;
    fn try_from(other: pluginv2::PluginContext) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: other.org_id,
            plugin_id: other.plugin_id,
            user: other.user.map(TryInto::try_into).transpose()?,
            app_instance_settings: other
                .app_instance_settings
                .map(TryInto::try_into)
                .transpose()?,
            datasource_instance_settings: other
                .data_source_instance_settings
                .map(TryInto::try_into)
                .transpose()?,
        })
    }
}
