//! SDK types and traits for checking health and collecting metrics from plugins.
use std::convert::{TryFrom, TryInto};

use serde_json::Value;

use crate::{
    backend::{ConversionError, PluginContext},
    pluginv2,
};

/// The health status of a plugin.
#[derive(Clone, Copy, Debug, PartialEq)]
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
pub struct CheckHealthRequest {
    /// Details of the plugin instance from which the request originated.
    pub plugin_context: Option<PluginContext>,
}

impl TryFrom<pluginv2::CheckHealthRequest> for CheckHealthRequest {
    type Error = ConversionError;
    fn try_from(other: pluginv2::CheckHealthRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other.plugin_context.map(TryInto::try_into).transpose()?,
        })
    }
}

/// The response to a health check request.
#[derive(Debug)]
pub struct CheckHealthResponse {
    /// The status of the plugin.
    pub status: HealthStatus,
    /// A message associated with the health check.
    pub message: String,
    /// Any additional details to include with the response.
    pub json_details: Value,
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
pub struct CollectMetricsRequest {
    /// Details of the plugin instance from which the request originated.
    pub plugin_context: Option<PluginContext>,
}

/// A response to a metric collection request.
///
/// This is unimplemented and subject to change.
// TODO: add proper Prometheus support.
pub struct CollectMetricsResponse {
    /// The metrics collected from Prometheus, in text exposition format.
    pub metrics: String,
}

/// Trait for services that provide a health check and/or metric collection endpoint.
///
/// The health check is used when checking that a datasource is configured correctly, or
/// (for app plugins) is exposed in Grafana's HTTP API.
///
/// Grafana will also expose a metrics endpoint at `/api/plugins/<plugin id>/metrics if this
/// trait is implemented, and will call the `collect_metrics` function to get metrics
/// for the plugin in text-based Prometheus exposition format. This allows plugins to be
/// instrumented in detail.
#[tonic::async_trait]
pub trait DiagnosticsService {
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
    /// See `<https://grafana.com/docs/grafana/latest/developers/plugins/backend/#health-checks>`.
    async fn check_health(
        &self,
        request: CheckHealthRequest,
    ) -> Result<CheckHealthResponse, Self::CheckHealthError>;

    /// The type of error that can occur when collecting metrics.
    type CollectMetricsError: std::error::Error;

    /// Collect metrics for a plugin.
    ///
    /// See `https://grafana.com/docs/grafana/latest/developers/plugins/backend/#collect-metrics`.
    async fn collect_metrics(&self) -> Result<CollectMetricsResponse, Self::CollectMetricsError>;
}

#[tonic::async_trait]
impl<T> pluginv2::diagnostics_server::Diagnostics for T
where
    T: DiagnosticsService + Send + Sync + 'static,
{
    async fn check_health(
        &self,
        request: tonic::Request<pluginv2::CheckHealthRequest>,
    ) -> Result<tonic::Response<pluginv2::CheckHealthResponse>, tonic::Status> {
        let response = DiagnosticsService::check_health(
            self,
            request
                .into_inner()
                .try_into()
                .map_err(ConversionError::into_tonic_status)?,
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(response.into()))
    }

    async fn collect_metrics(
        &self,
        _request: tonic::Request<pluginv2::CollectMetricsRequest>,
    ) -> Result<tonic::Response<pluginv2::CollectMetricsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented(
            "Metric collection is not yet implemented in the Rust SDK.",
        ))
    }
}
