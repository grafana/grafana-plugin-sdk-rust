use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    io,
    net::SocketAddr,
};

use chrono::prelude::*;
use serde_json::Value;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::pluginv2;

mod data;
mod diagnostics;
mod stream;

pub use data::{
    BoxQueryDataResponse, DataQuery, DataQueryError, DataResponse, DataService, QueryDataRequest,
};
pub use diagnostics::{
    CheckHealthRequest, CheckHealthResponse, CollectMetricsRequest, CollectMetricsResponse,
    DiagnosticsService, HealthStatus,
};
pub use stream::{
    PublishStreamRequest, PublishStreamResponse, RunStreamRequest, StreamPacket, StreamService,
    SubscribeStreamRequest, SubscribeStreamResponse, SubscribeStreamStatus,
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
    let address = TcpListener::bind("[::1]:0").await?.local_addr()?;
    println!("1|2|tcp|{}|grpc", address);
    Ok(address)
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("time_range missing from query")]
    MissingTimeRange,
    #[error("plugin_context missing from request")]
    MissingPluginContext,
    #[error("invalid JSON (got {json})")]
    InvalidJson {
        err: serde_json::Error,
        json: String,
    },
    #[error("invalid frame")]
    InvalidFrame { source: serde_json::Error },
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

#[derive(Debug)]
pub struct User {
    pub login: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

impl From<pluginv2::User> for User {
    fn from(other: pluginv2::User) -> Self {
        Self {
            login: other.login,
            name: other.name,
            email: other.email,
            role: other.role,
        }
    }
}

#[derive(Debug)]
pub struct AppInstanceSettings {
    pub json_data: Value,
    pub decrypted_secure_json_data: HashMap<String, String>,
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

#[derive(Debug)]
pub struct DataSourceInstanceSettings {
    // The Grafana assigned numeric identifier of the the data source instance.
    pub id: i64,

    /// The Grafana assigned string identifier of the the data source instance.
    pub uid: String,

    /// The configured name of the data source instance.
    pub name: String,

    /// The configured URL of a data source instance (e.g. the URL of an API endpoint).
    pub url: String,

    /// A configured user for a data source instance. This is not a Grafana user, rather an arbitrary string.
    pub user: String,

    /// The configured database for a data source instance. (e.g. the default Database a SQL data source would connect to).
    pub database: String,

    /// BasicAuthEnabled indicates if this data source instance should use basic authentication.
    pub basic_auth_enabled: bool,

    /// The configured user for basic authentication.
    ///
    /// E.g. when a data source uses basic authentication to connect to whatever API it fetches data from).
    pub basic_auth_user: String,

    /// The raw DataSourceConfig as JSON as stored by Grafana server.
    ///
    /// It repeats the properties in this object and includes custom properties.
    pub json_data: Value,

    /// Key:value pairs where the encrypted configuration in Grafana server have been
    /// decrypted before passing them to the plugin.
    pub decrypted_secure_json_data: HashMap<String, String>,

    /// The last time the configuration for the data source instance was updated.
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

#[derive(Debug)]
pub struct PluginContext {
    pub org_id: i64,
    pub plugin_id: String,
    pub user: Option<User>,
    pub app_instance_settings: Option<AppInstanceSettings>,
    pub data_source_instance_settings: Option<DataSourceInstanceSettings>,
}

impl TryFrom<pluginv2::PluginContext> for PluginContext {
    type Error = ConversionError;
    fn try_from(other: pluginv2::PluginContext) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: other.org_id,
            plugin_id: other.plugin_id,
            user: other.user.map(Into::into),
            app_instance_settings: other
                .app_instance_settings
                .map(TryInto::try_into)
                .transpose()?,
            data_source_instance_settings: other
                .data_source_instance_settings
                .map(TryInto::try_into)
                .transpose()?,
        })
    }
}
