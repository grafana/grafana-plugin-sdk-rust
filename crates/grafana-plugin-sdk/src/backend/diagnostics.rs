//! SDK types and traits for checking health and collecting metrics from plugins.

use std::{collections::HashMap, fmt};

use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::instrument;

use crate::{
    backend::{ConvertFromError, InstanceSettings, PluginContext},
    pluginv2,
};

use super::{GrafanaPlugin, PluginType};

/// The health status of a plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HealthStatus {
    /// The plugin was unable to determine if it was healthy.
    Unknown,
    /// The plugin is working as expected.
    Ok,
    /// The plugin is in an error state.
    Error,
}

impl From<HealthStatus> for pluginv2::check_health_response::HealthStatus {
    fn from(other: HealthStatus) -> Self {
        match other {
            HealthStatus::Unknown => pluginv2::check_health_response::HealthStatus::Unknown,
            HealthStatus::Ok => pluginv2::check_health_response::HealthStatus::Ok,
            HealthStatus::Error => pluginv2::check_health_response::HealthStatus::Error,
        }
    }
}

/// A request to check the health of a plugin.
#[derive(Debug)]
#[non_exhaustive]
pub struct InnerCheckHealthRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    /// Details of the plugin instance from which the request originated.
    pub plugin_context: PluginContext<IS, JsonData, SecureJsonData>,
    /// Headers included along with the request by Grafana.
    pub headers: HashMap<String, String>,
}

impl<IS, JsonData, SecureJsonData> TryFrom<pluginv2::CheckHealthRequest>
    for InnerCheckHealthRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    type Error = ConvertFromError;
    #[instrument(err)]
    fn try_from(other: pluginv2::CheckHealthRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConvertFromError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            headers: other.headers,
        })
    }
}

/// A request to check the health of a plugin.
///
/// This is a convenience type alias to hide some of the complexity of
/// the various generics involved.
///
/// The type parameter `T` is the type of the plugin implementation itself,
/// which must implement `GrafanaPlugin`.
pub type CheckHealthRequest<T> = InnerCheckHealthRequest<
    <<T as GrafanaPlugin>::PluginType as PluginType<
        <T as GrafanaPlugin>::JsonData,
        <T as GrafanaPlugin>::SecureJsonData,
    >>::InstanceSettings,
    <T as GrafanaPlugin>::JsonData,
    <T as GrafanaPlugin>::SecureJsonData,
>;

/// The response to a health check request.
#[derive(Debug)]
#[non_exhaustive]
pub struct CheckHealthResponse {
    /// The status of the plugin.
    pub status: HealthStatus,
    /// A message associated with the health check.
    pub message: String,
    /// Any additional details to include with the response.
    pub json_details: Value,
}

impl CheckHealthResponse {
    /// Create a new `CheckHealthResponse`.
    #[deprecated(since = "1.3.0", note = "use ok/error/unknown constructors instead")]
    pub fn new(status: HealthStatus, message: String, json_details: Value) -> Self {
        Self {
            status,
            message,
            json_details,
        }
    }

    /// Create a `CheckHealthResponse` with status [`HealthStatus::Ok`].
    ///
    /// The JSON in `json_details` will be set to `null`; use [`CheckHealthResponse::with_json_details`]
    /// to override it.
    pub fn ok(message: String) -> Self {
        Self {
            status: HealthStatus::Ok,
            message,
            json_details: Value::Null,
        }
    }

    /// Create a `CheckHealthResponse` with status [`HealthStatus::Error`].
    ///
    /// The JSON in `json_details` will be set to `null`; use [`CheckHealthResponse::with_json_details`]
    /// to override it.
    pub fn error(message: String) -> Self {
        Self {
            status: HealthStatus::Error,
            message,
            json_details: Value::Null,
        }
    }

    /// Create a `CheckHealthResponse` with status [`HealthStatus::Unknown`].
    ///
    /// The JSON in `json_details` will be set to `null`; use [`CheckHealthResponse::with_json_details`]
    /// to override it.
    pub fn unknown(message: String) -> Self {
        Self {
            status: HealthStatus::Unknown,
            message,
            json_details: Value::Null,
        }
    }

    /// Update `self` with the given JSON details, returning a new `CheckHealthResponse`.
    pub fn with_json_details(mut self, json_details: Value) -> Self {
        self.json_details = json_details;
        self
    }
}

impl Default for CheckHealthResponse {
    fn default() -> Self {
        Self {
            status: HealthStatus::Ok,
            message: "OK".to_string(),
            json_details: Default::default(),
        }
    }
}

impl From<CheckHealthResponse> for pluginv2::CheckHealthResponse {
    fn from(other: CheckHealthResponse) -> Self {
        let mut response = pluginv2::CheckHealthResponse {
            status: 0_i32,
            message: other.message,
            json_details: serde_json::to_vec(&other.json_details)
                .expect("Value contained invalid JSON"),
        };
        response.set_status(other.status.into());
        response
    }
}

/// A request to collect metrics about a plugin.
#[derive(Debug)]
#[non_exhaustive]
pub struct InnerCollectMetricsRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    /// Details of the plugin instance from which the request originated.
    pub plugin_context: PluginContext<IS, JsonData, SecureJsonData>,
}

impl<IS, JsonData, SecureJsonData> TryFrom<pluginv2::CollectMetricsRequest>
    for InnerCollectMetricsRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::CollectMetricsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConvertFromError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
        })
    }
}

/// A request to collect metrics about a plugin.
///
/// This is a convenience type alias to hide some of the complexity of
/// the various generics involved.
///
/// The type parameter `T` is the type of the plugin implementation itself,
/// which must implement `GrafanaPlugin`.
pub type CollectMetricsRequest<T> = InnerCollectMetricsRequest<
    <<T as GrafanaPlugin>::PluginType as PluginType<
        <T as GrafanaPlugin>::JsonData,
        <T as GrafanaPlugin>::SecureJsonData,
    >>::InstanceSettings,
    <T as GrafanaPlugin>::JsonData,
    <T as GrafanaPlugin>::SecureJsonData,
>;

/// Metrics collected from a plugin as part of a collect metrics.
#[derive(Debug)]
#[non_exhaustive]
pub struct Payload {
    /// The metrics, in Prometheus text exposition format.
    pub prometheus: Vec<u8>,
}

impl Payload {
    /// Create a payload containing Prometheus metrics, in Prometheus text exposition format.
    pub fn prometheus(bytes: Vec<u8>) -> Self {
        Self { prometheus: bytes }
    }
}

impl From<Payload> for pluginv2::collect_metrics_response::Payload {
    fn from(other: Payload) -> Self {
        Self {
            prometheus: other.prometheus,
        }
    }
}

/// A response to a metric collection request.
#[derive(Debug)]
#[non_exhaustive]
pub struct CollectMetricsResponse {
    /// The metrics collected from the plugin.
    pub metrics: Option<Payload>,
}

impl CollectMetricsResponse {
    /// Create a new `CollectMetricsResponse`.
    pub fn new(metrics: Option<Payload>) -> Self {
        Self { metrics }
    }
}

impl From<CollectMetricsResponse> for pluginv2::CollectMetricsResponse {
    fn from(other: CollectMetricsResponse) -> Self {
        Self {
            metrics: other.metrics.map(Into::into),
        }
    }
}

/// Trait for services that provide a health check and/or metric collection endpoint.
///
/// The health check is used when checking that a datasource is configured correctly, or
/// (for app plugins) is exposed in Grafana's HTTP API.
///
/// Grafana will also expose a metrics endpoint at `/api/plugins/<plugin id>/metrics` if this
/// trait is implemented, and will call the `collect_metrics` function to get metrics
/// for the plugin in text-based Prometheus exposition format. This allows plugins to be
/// instrumented in detail.
///
/// # Example
///
/// ```rust
/// use grafana_plugin_sdk::{backend, prelude::*};
/// use prometheus::{Encoder, TextEncoder};
///
/// #[derive(Clone, Debug, GrafanaPlugin)]
/// #[grafana_plugin(plugin_type = "app")]
/// struct MyPlugin {
///     metrics: prometheus::Registry,
/// }
///
/// #[backend::async_trait]
/// impl backend::DiagnosticsService for MyPlugin {
///     type CheckHealthError = std::convert::Infallible;
///
///     async fn check_health(
///         &self,
///         request: backend::CheckHealthRequest<Self>,
///     ) -> Result<backend::CheckHealthResponse, Self::CheckHealthError> {
///         // A real plugin may ensure it could e.g. connect to a database, was configured
///         // correctly, etc.
///         Ok(backend::CheckHealthResponse::ok("Ok".to_string()))
///     }
///
///     type CollectMetricsError = prometheus::Error;
///
///     async fn collect_metrics(
///         &self,
///         request: backend::CollectMetricsRequest<Self>,
///     ) -> Result<backend::CollectMetricsResponse, Self::CollectMetricsError> {
///         let mut buffer = vec![];
///         let encoder = TextEncoder::new();
///         encoder.encode(&self.metrics.gather(), &mut buffer)?;
///         Ok(backend::CollectMetricsResponse::new(Some(backend::MetricsPayload::prometheus(buffer))))
///     }
/// }
/// ```
#[tonic::async_trait]
pub trait DiagnosticsService: GrafanaPlugin {
    /// The type of error that can occur when performing a health check request.
    type CheckHealthError: std::error::Error;

    /// Check the health of a plugin.
    ///
    /// For a datasource plugin, this is called automatically when a user clicks 'Save & Test'
    /// in the UI when editing a datasource.
    ///
    /// For an app plugin, a health check endpoint is exposed in the Grafana HTTP API and
    /// allows external systems to poll the plugin's health to make sure it is running as expected.
    ///
    /// See <https://grafana.com/docs/grafana/latest/developers/plugins/backend/#health-checks>.
    async fn check_health(
        &self,
        request: CheckHealthRequest<Self>,
    ) -> Result<CheckHealthResponse, Self::CheckHealthError>;

    /// The type of error that can occur when collecting metrics.
    type CollectMetricsError: std::error::Error;

    /// Collect metrics for a plugin.
    ///
    /// See <https://grafana.com/docs/grafana/latest/developers/plugins/backend/#collect-metrics>.
    async fn collect_metrics(
        &self,
        request: CollectMetricsRequest<Self>,
    ) -> Result<CollectMetricsResponse, Self::CollectMetricsError>;
}

#[tonic::async_trait]
impl<T> pluginv2::diagnostics_server::Diagnostics for T
where
    T: DiagnosticsService + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self), level = "debug")]
    async fn check_health(
        &self,
        request: tonic::Request<pluginv2::CheckHealthRequest>,
    ) -> Result<tonic::Response<pluginv2::CheckHealthResponse>, tonic::Status> {
        let response = match request.into_inner().try_into() {
            Ok(request) => DiagnosticsService::check_health(self, request)
                .await
                .unwrap_or_else(|e| CheckHealthResponse::error(e.to_string())),
            Err(e) => {
                CheckHealthResponse::error(format!("error converting check health request: {e}"))
                    .with_json_details(serde_json::to_value(e).unwrap_or(serde_json::Value::Null))
            }
        };
        Ok(tonic::Response::new(response.into()))
    }

    #[tracing::instrument(skip(self), level = "debug")]
    async fn collect_metrics(
        &self,
        request: tonic::Request<pluginv2::CollectMetricsRequest>,
    ) -> Result<tonic::Response<pluginv2::CollectMetricsResponse>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConvertFromError::into_tonic_status)?;
        let response = DiagnosticsService::collect_metrics(self, request)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(response.into()))
    }
}
