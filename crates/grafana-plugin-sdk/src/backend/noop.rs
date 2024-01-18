//! No-op service implementations for each plugin service.
use std::convert::Infallible;

use super::*;

/// A no-op service which implements each of the available service traits.
///
/// This is used as a default type parameter of [`crate::backend::Plugin`]
/// to enable a more type-safe API. The `Plugin` starts off in a state where
/// every `service` field is assumed to be an optional `NoopService`;
/// calling the `*_service()` methods constructs a new `Plugin` with type
/// parameterized by the provided service, which replaces this struct.
/// We can then use tonic's [`add_optional_service`][tonic::transport::server::Router::add_optional_service]
/// to avoid actual having to construct this struct.
///
/// This struct cannot be constructed, so
/// the `unreachable`s in this module should never be reached.
#[derive(Debug)]
pub struct NoopService {
    _priv: (),
}

impl DataQueryError for Infallible {
    fn ref_id(self) -> String {
        unreachable!()
    }
}

impl GrafanaPlugin for NoopService {
    type PluginType = AppPlugin<Self::JsonData, Self::SecureJsonData>;
    type JsonData = Value;
    type SecureJsonData = HashMap<String, String>;
}

#[tonic::async_trait]
impl DataService for NoopService {
    type Query = ();
    type QueryError = Infallible;
    type Stream = BoxDataResponseStream<Self::QueryError>;
    async fn query_data(&self, _request: QueryDataRequest<Self::Query, Self>) -> Self::Stream {
        unreachable!()
    }
}

#[tonic::async_trait]
impl DiagnosticsService for NoopService {
    type CheckHealthError = Infallible;
    async fn check_health(
        &self,
        _request: CheckHealthRequest<Self>,
    ) -> Result<CheckHealthResponse, Self::CheckHealthError> {
        unreachable!()
    }
    type CollectMetricsError = Infallible;
    async fn collect_metrics(
        &self,
        _request: CollectMetricsRequest<Self>,
    ) -> Result<CollectMetricsResponse, Self::CollectMetricsError> {
        unreachable!()
    }
}

#[tonic::async_trait]
impl ResourceService for NoopService {
    type Error = Infallible;
    type InitialResponse = Vec<u8>;
    type Stream = BoxResourceStream<Self::Error>;

    /// Handle a resource request.
    ///
    /// It is completely up to the implementor how to handle the incoming request.
    ///
    /// A stream of responses can be returned. A simple way to return just a single response
    /// is to use `futures_util::stream::once`.
    async fn call_resource(
        &self,
        _request: CallResourceRequest<Self>,
    ) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
        unreachable!()
    }
}

#[tonic::async_trait]
impl StreamService for NoopService {
    async fn subscribe_stream(
        &self,
        _request: SubscribeStreamRequest<Self>,
    ) -> Result<SubscribeStreamResponse, Self::Error> {
        unreachable!()
    }
    type JsonValue = ();
    type Error = Infallible;
    type Stream = BoxRunStream<Self::Error>;
    async fn run_stream(
        &self,
        _request: RunStreamRequest<Self>,
    ) -> Result<Self::Stream, Self::Error> {
        unreachable!()
    }
    async fn publish_stream(
        &self,
        _request: PublishStreamRequest<Self>,
    ) -> Result<PublishStreamResponse, Self::Error> {
        unreachable!()
    }
}
