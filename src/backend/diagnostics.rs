use std::convert::{TryFrom, TryInto};

use serde_json::Value;

use crate::{
    backend::{ConversionError, PluginContext},
    pluginv2,
};

pub enum HealthStatus {
    Unknown,
    Ok,
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

pub struct CheckHealthRequest {
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

pub struct CheckHealthResponse {
    pub status: HealthStatus,
    pub message: String,
    pub json_details: Value,
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

pub struct CollectMetricsRequest {
    pub plugin_context: Option<PluginContext>,
}

pub struct CollectMetricsResponse {
    // TODO: add Prometheus support
    pub metrics: Vec<String>,
}

#[tonic::async_trait]
pub trait DiagnosticsService {
    type CheckHealthError: std::error::Error;
    async fn check_health(
        &self,
        request: CheckHealthRequest,
    ) -> Result<CheckHealthResponse, Self::CheckHealthError>;
    type CollectMetricsError: std::error::Error;
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
