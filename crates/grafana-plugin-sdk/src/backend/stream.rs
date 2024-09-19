//! SDK types and traits relevant to plugins that stream data.
use std::{fmt, pin::Pin};

use futures_util::{Stream, StreamExt, TryStreamExt};
use prost::bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    backend::{ConvertFromError, ConvertToError, InstanceSettings, PluginContext},
    data,
    live::Path,
    pluginv2,
};

use super::{GrafanaPlugin, PluginType};

/// A request to subscribe to a stream.
#[derive(Debug)]
#[non_exhaustive]
pub struct InnerSubscribeStreamRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    /// Details of the plugin instance from which the request originated.
    ///
    /// If the request originates from a datasource instance, this will
    /// include details about the datasource instance in the
    /// `data_source_instance_settings` field.
    pub plugin_context: PluginContext<IS, JsonData, SecureJsonData>,

    /// The subscription channel path that the request wishes to subscribe to.
    pub path: Path,

    /// Optional raw data.
    ///
    /// This may be used as an extra payload supplied upon subscription;
    /// for example, this may contain a JSON query object. This will be
    /// empty if not supplied in the query.
    pub data: Bytes,
}

impl<IS, JsonData, SecureJsonData> TryFrom<pluginv2::SubscribeStreamRequest>
    for InnerSubscribeStreamRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::SubscribeStreamRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConvertFromError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            path: Path::new(other.path)?,
            data: other.data,
        })
    }
}

/// A request to 'run' a stream, i.e. begin streaming data.
///
/// This is made by Grafana _after_ a stream subscription request has been accepted,
/// and will include the same `path` as the subscription request.
///
/// This is a convenience type alias to hide some of the complexity of
/// the various generics involved.
///
/// The type parameter `T` is the type of the plugin implementation itself,
/// which must implement [`GrafanaPlugin`].
pub type SubscribeStreamRequest<T> = InnerSubscribeStreamRequest<
    <<T as GrafanaPlugin>::PluginType as PluginType<
        <T as GrafanaPlugin>::JsonData,
        <T as GrafanaPlugin>::SecureJsonData,
    >>::InstanceSettings,
    <T as GrafanaPlugin>::JsonData,
    <T as GrafanaPlugin>::SecureJsonData,
>;

/// The status of a subscribe stream response.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum SubscribeStreamStatus {
    /// The request to subscribe was accepted.
    Ok,
    /// The requested path was not found.
    NotFound,
    /// The user did not have permission to subscribe to the requested stream.
    PermissionDenied,
}

impl From<SubscribeStreamStatus> for pluginv2::subscribe_stream_response::Status {
    fn from(other: SubscribeStreamStatus) -> Self {
        match other {
            SubscribeStreamStatus::Ok => Self::Ok,
            SubscribeStreamStatus::NotFound => Self::NotFound,
            SubscribeStreamStatus::PermissionDenied => Self::PermissionDenied,
        }
    }
}

/// Data returned from an initial request to subscribe to a stream.
#[derive(Debug)]
pub struct InitialData {
    data: Vec<u8>,
}

impl InitialData {
    /// Create some initial data representing a `Frame`.
    pub fn from_frame(
        frame: data::CheckedFrame<'_>,
        include: data::FrameInclude,
    ) -> Result<Self, ConvertToError> {
        Ok(Self {
            data: frame
                .to_json(include)
                .map_err(|source| ConvertToError::InvalidFrame { source })?,
        })
    }

    /// Create some initial data representing some JSON.
    pub fn from_json<T>(json: &T) -> Result<Self, ConvertToError>
    where
        T: Serialize,
    {
        Ok(Self {
            data: serde_json::to_vec(json).map_err(|err| ConvertToError::InvalidJson { err })?,
        })
    }
}

/// The response to a stream subscription request.
///
/// This includes a status and some optional initial data for the stream.
#[derive(Debug)]
#[non_exhaustive]
pub struct SubscribeStreamResponse {
    /// The status of the response.
    pub status: SubscribeStreamStatus,
    /// Optional initial data to return to the client, used to pre-populate the stream.
    pub initial_data: Option<InitialData>,
}

impl SubscribeStreamResponse {
    /// Create a new `SubscribeStreamResponse`.
    #[deprecated(
        since = "1.3.0",
        note = "use ok/not_found/permission_denied constructors instead"
    )]
    pub fn new(status: SubscribeStreamStatus, initial_data: Option<InitialData>) -> Self {
        Self {
            status,
            initial_data,
        }
    }

    /// Create a `SubscribeStreamResponse` with status [`SubscribeStreamStatus::Ok`].
    ///
    /// This is the happy path to be used when a subscription request succeeded.
    pub fn ok(initial_data: Option<InitialData>) -> Self {
        Self {
            status: SubscribeStreamStatus::Ok,
            initial_data,
        }
    }

    /// Create a `SubscribeStreamResponse` with status [`SubscribeStreamStatus::NotFound`].
    ///
    /// This should be returned when the caller requested an unknown path.
    pub fn not_found() -> Self {
        Self {
            status: SubscribeStreamStatus::NotFound,
            initial_data: None,
        }
    }

    /// Create a `SubscribeStreamResponse` with status [`SubscribeStreamStatus::PermissionDenied`].
    ///
    /// This should be returned when the caller is not permitted to access the requested path.
    pub fn permission_denied() -> Self {
        Self {
            status: SubscribeStreamStatus::PermissionDenied,
            initial_data: None,
        }
    }
}

impl From<SubscribeStreamResponse> for pluginv2::SubscribeStreamResponse {
    fn from(other: SubscribeStreamResponse) -> Self {
        let mut response = pluginv2::SubscribeStreamResponse {
            status: 0,
            data: other.initial_data.map(|x| x.data).unwrap_or_default(),
        };
        response.set_status(other.status.into());
        response
    }
}

/// A request to 'run' a stream, i.e. begin streaming data.
///
/// This is made by Grafana _after_ a stream subscription request has been accepted,
/// and will include the same `path` as the subscription request.
#[derive(Debug)]
#[non_exhaustive]
pub struct InnerRunStreamRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    /// Metadata about the plugin from which the request originated.
    pub plugin_context: PluginContext<IS, JsonData, SecureJsonData>,

    /// The subscription path; see module level comments for details.
    pub path: Path,

    /// Optional raw data.
    ///
    /// This may be used as an extra payload supplied upon subscription;
    /// for example, this may contain a JSON query object. This will be
    /// empty if not supplied in the query.
    pub data: Bytes,
}

impl<IS, JsonData, SecureJsonData> TryFrom<pluginv2::RunStreamRequest>
    for InnerRunStreamRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::RunStreamRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConvertFromError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            path: Path::new(other.path)?,
            data: other.data,
        })
    }
}

/// A request to 'run' a stream, i.e. begin streaming data.
///
/// This is made by Grafana _after_ a stream subscription request has been accepted,
/// and will include the same `path` as the subscription request.
///
/// This is a convenience type alias to hide some of the complexity of
/// the various generics involved.
///
/// The type parameter `T` is the type of the plugin implementation itself,
/// which must implement [`GrafanaPlugin`].
pub type RunStreamRequest<T> = InnerRunStreamRequest<
    <<T as GrafanaPlugin>::PluginType as PluginType<
        <T as GrafanaPlugin>::JsonData,
        <T as GrafanaPlugin>::SecureJsonData,
    >>::InstanceSettings,
    <T as GrafanaPlugin>::JsonData,
    <T as GrafanaPlugin>::SecureJsonData,
>;

/// A packet of data to be streamed back to the subscribed client.
///
/// Such data can be:
/// - a [`Frame`][data::Frame], which will be serialized to JSON before being sent back to the client
/// - arbitrary JSON
/// - arbitrary bytes.
///
/// The `J` type parameter on this enum is only relevant when JSON data
/// is being streamed back,
#[derive(Debug)]
pub struct StreamPacket<J = ()> {
    data: Vec<u8>,
    _p: std::marker::PhantomData<J>,
}

impl<J> StreamPacket<J> {
    /// Create a `StreamPacket` representing a `Frame`.
    pub fn from_frame(frame: data::CheckedFrame<'_>) -> Result<Self, ConvertToError> {
        Ok(Self {
            data: frame
                .to_json(data::FrameInclude::All)
                .map_err(|source| ConvertToError::InvalidFrame { source })?,
            _p: std::marker::PhantomData,
        })
    }

    /// Create a `StreamPacket` representing some JSON.
    pub fn from_json(json: &J) -> Result<Self, ConvertToError>
    where
        J: Serialize,
    {
        Ok(Self {
            data: serde_json::to_vec(json).map_err(|err| ConvertToError::InvalidJson { err })?,
            _p: std::marker::PhantomData,
        })
    }

    /// Create a `StreamPacket` from arbitrary bytes.
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            _p: std::marker::PhantomData,
        }
    }

    fn into_plugin_packet(self) -> pluginv2::StreamPacket {
        pluginv2::StreamPacket { data: self.data }
    }
}

/// Type alias for a pinned, boxed stream of stream packets with a custom error type.
pub type BoxRunStream<E, T = ()> = Pin<Box<dyn Stream<Item = Result<StreamPacket<T>, E>> + Send>>;

/// A request to publish data to a stream.
#[non_exhaustive]
pub struct InnerPublishStreamRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    /// Details of the plugin instance from which the request originated.
    ///
    /// If the request originates from a datasource instance, this will
    /// include details about the datasource instance in the
    /// `data_source_instance_settings` field.
    pub plugin_context: PluginContext<IS, JsonData, SecureJsonData>,
    /// The subscription path; see module level comments for details.
    pub path: Path,
    /// Data to be published to the stream.
    pub data: serde_json::Value,
}

impl<IS, JsonData, SecureJsonData> TryFrom<pluginv2::PublishStreamRequest>
    for InnerPublishStreamRequest<IS, JsonData, SecureJsonData>
where
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::PublishStreamRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConvertFromError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            path: Path::new(other.path)?,
            data: super::read_json_query(&other.data)?,
        })
    }
}

/// A request to publish data to a stream.
///
/// This is a convenience type alias to hide some of the complexity of
/// the various generics involved.
///
/// The type parameter `T` is the type of the plugin implementation itself,
/// which must implement [`GrafanaPlugin`].
pub type PublishStreamRequest<T> = InnerPublishStreamRequest<
    <<T as GrafanaPlugin>::PluginType as PluginType<
        <T as GrafanaPlugin>::JsonData,
        <T as GrafanaPlugin>::SecureJsonData,
    >>::InstanceSettings,
    <T as GrafanaPlugin>::JsonData,
    <T as GrafanaPlugin>::SecureJsonData,
>;

/// The status of a publish stream response.
#[non_exhaustive]
pub enum PublishStreamStatus {
    /// The request to publish was accepted.
    Ok,
    /// The requested path was not found.
    NotFound,
    /// The user did not have permission to publish to the requested stream.
    PermissionDenied,
}

impl From<PublishStreamStatus> for pluginv2::publish_stream_response::Status {
    fn from(other: PublishStreamStatus) -> Self {
        match other {
            PublishStreamStatus::Ok => Self::Ok,
            PublishStreamStatus::NotFound => Self::NotFound,
            PublishStreamStatus::PermissionDenied => Self::PermissionDenied,
        }
    }
}

/// The response to a stream publish request.
#[non_exhaustive]
pub struct PublishStreamResponse {
    /// The status of the response.
    pub status: PublishStreamStatus,
    /// Data returned in response to publishing.
    pub data: serde_json::Value,
}

impl PublishStreamResponse {
    /// Create a new `PublishStreamResponse`.
    #[deprecated(
        since = "1.3.0",
        note = "use ok/not_found/permission_denied constructors instead"
    )]
    pub fn new(status: PublishStreamStatus, data: serde_json::Value) -> Self {
        Self { status, data }
    }

    /// Create a `PublishStreamResponse` with status [`PublishStreamStatus::Ok`].
    ///
    /// This is the happy path to be used when a publish request succeeded.
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            status: PublishStreamStatus::Ok,
            data,
        }
    }

    /// Create a `PublishStreamResponse` with status [`PublishStreamStatus::NotFound`].
    ///
    /// This should be returned when the caller requested an unknown path.
    pub fn not_found(details: serde_json::Value) -> Self {
        Self {
            status: PublishStreamStatus::NotFound,
            data: details,
        }
    }

    /// Create a `PublishStreamResponse` with status [`PublishStreamStatus::PermissionDenied`].
    ///
    /// This should be returned when the caller is not permitted to access the requested path.
    pub fn permission_denied(details: serde_json::Value) -> Self {
        Self {
            status: PublishStreamStatus::PermissionDenied,
            data: details,
        }
    }
}

impl TryFrom<PublishStreamResponse> for pluginv2::PublishStreamResponse {
    type Error = serde_json::Error;
    fn try_from(other: PublishStreamResponse) -> Result<Self, Self::Error> {
        let mut response = pluginv2::PublishStreamResponse {
            status: 0,
            data: serde_json::to_vec(&other.data)?,
        };
        response.set_status(other.status.into());
        Ok(response)
    }
}

/// Trait for plugins that wish to provide uni- or bi-directional streaming.
///
/// # Example
///
/// ```rust
/// use std::{sync::Arc, time::Duration};
///
/// use grafana_plugin_sdk::{backend, data, prelude::*};
/// use thiserror::Error;
/// use tokio::sync::RwLock;
/// use tokio_stream::StreamExt;
/// use tracing::{debug, info};
///
/// #[derive(Clone, Debug, GrafanaPlugin)]
/// #[grafana_plugin(plugin_type = "datasource")]
/// struct MyPlugin;
///
/// #[derive(Debug, Error)]
/// #[error("Error streaming data")]
/// struct StreamError;
///
/// impl From<data::Error> for StreamError {
///     fn from(_other: data::Error) -> StreamError {
///         StreamError
///     }
/// }
///
/// impl From<backend::ConvertToError> for StreamError {
///     fn from(_other: backend::ConvertToError) -> StreamError {
///         StreamError
///     }
/// }
///
/// #[backend::async_trait]
/// impl backend::StreamService for MyPlugin {
///     /// The type of JSON value we might return in our `initial_data`.
///     ///
///     /// If we're not returning JSON we can just use `()`.
///     type JsonValue = ();
///
///     /// Handle a request to subscribe to a stream.
///     ///
///     /// Here we just check that the path matches some fixed value
///     /// and return `NotFound` if not.
///     async fn subscribe_stream(
///         &self,
///         request: backend::SubscribeStreamRequest<Self>,
///     ) -> Result<backend::SubscribeStreamResponse, Self::Error> {
///         let response = if request.path.as_str() == "stream" {
///             backend::SubscribeStreamResponse::ok(None)
///         } else {
///             backend::SubscribeStreamResponse::not_found()
///         };
///         info!(path = %request.path, "Subscribing to stream");
///         Ok(response)
///     }
///
///     type Error = StreamError;
///     type Stream = backend::BoxRunStream<Self::Error>;
///
///     /// Begin streaming data for a request.
///     ///
///     /// This example just creates an in-memory `Frame` in each loop iteration,
///     /// sends an updated version of the frame once per second, and updates a loop variable
///     /// so that each frame is different.
///     async fn run_stream(&self, _request: backend::RunStreamRequest<Self>) -> Result<Self::Stream, Self::Error> {
///         info!("Running stream");
///         let mut x = 0u32;
///         let n = 3;
///         let mut frame = data::Frame::new("foo").with_field((x..x + n).into_field("x"));
///         Ok(Box::pin(
///             async_stream::try_stream! {
///                 loop {
///                     frame.fields_mut()[0].set_values(x..x + n);
///                     let packet = backend::StreamPacket::from_frame(frame.check()?)?;
///                     debug!("Yielding frame from {} to {}", x, x + n);
///                     yield packet;
///                     x += n;
///                 }
///             }
///             .throttle(Duration::from_secs(1)),
///         ))
///     }
///
///     /// Handle a request to publish data to a stream.
///     ///
///     /// Currently unimplemented in this example, but the functionality _should_ work.
///     async fn publish_stream(
///         &self,
///         _request: backend::PublishStreamRequest<Self>,
///     ) -> Result<backend::PublishStreamResponse, Self::Error> {
///         info!("Publishing to stream");
///         todo!()
///     }
/// }
/// ```
#[tonic::async_trait]
pub trait StreamService: GrafanaPlugin {
    /// Handle requests to begin a subscription to a plugin or datasource managed channel path.
    ///
    ///
    /// This function is called for _every_ subscriber to a stream.  Implementations should
    /// check the subscribe permissions of the incoming request, and can choose to return some
    /// initial data to prepopulate the stream.
    ///
    /// `run_stream` will generally be called shortly after returning a response with
    /// [`SubscribeStreamStatus::Ok`]; this is responsible for streaming any data after
    /// the [`initial_data`][SubscribeStreamResponse::initial_data].
    async fn subscribe_stream(
        &self,
        request: SubscribeStreamRequest<Self>,
    ) -> Result<SubscribeStreamResponse, Self::Error>;

    /// The type of JSON values returned by this stream service.
    ///
    /// Each [`StreamPacket`] can return either a [`data::Frame`] or some arbitary JSON. This
    /// associated type allows the JSON value to be statically typed, if desired.
    ///
    /// If the implementation does not intend to return JSON variants, this
    /// can be set to `()`. If the structure of the returned JSON is not statically known, this
    /// should be set to [`serde_json::Value`].
    type JsonValue: Serialize;

    /// The type of error that can occur while fetching a stream packet.
    type Error: std::error::Error;

    /// The type of stream returned by `run_stream`.
    ///
    /// This will generally be impossible to name directly, so returning the
    /// [`BoxRunStream`] type alias will probably be more convenient.
    type Stream: futures_core::Stream<Item = Result<StreamPacket<Self::JsonValue>, Self::Error>>
        + Send;

    /// Begin sending stream packets to a client.
    ///
    /// This will only be called once per channel, shortly after the first successful subscription
    /// to that channel by the first client (after `subscribe_stream` returns a response with
    /// [`SubscribeStreamStatus::Ok`] for a specific [`Channel`][crate::live::Channel]).
    /// Grafana will then multiplex the returned stream to any future subscribers.
    ///
    /// When Grafana detects that there are no longer any subscribers to a channel, the stream
    /// will be terminated until the next active subscriber appears. Stream termination can
    /// may be slightly delayed, generally by a few seconds.
    async fn run_stream(
        &self,
        request: RunStreamRequest<Self>,
    ) -> Result<Self::Stream, Self::Error>;

    /// Handle requests to publish to a plugin or datasource managed channel path (currently unimplemented).
    ///
    /// Implementations should check the publish permissions of the incoming request.
    async fn publish_stream(
        &self,
        request: PublishStreamRequest<Self>,
    ) -> Result<PublishStreamResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> pluginv2::stream_server::Stream for T
where
    T: Send + Sync + StreamService + 'static,
{
    #[tracing::instrument(skip(self), level = "debug")]
    async fn subscribe_stream(
        &self,
        request: tonic::Request<pluginv2::SubscribeStreamRequest>,
    ) -> Result<tonic::Response<pluginv2::SubscribeStreamResponse>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConvertFromError::into_tonic_status)?;
        let response = StreamService::subscribe_stream(self, request)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .into();
        Ok(tonic::Response::new(response))
    }

    type RunStreamStream = Pin<
        Box<dyn futures_core::Stream<Item = Result<pluginv2::StreamPacket, tonic::Status>> + Send>,
    >;

    #[tracing::instrument(skip(self), level = "debug")]
    async fn run_stream(
        &self,
        request: tonic::Request<pluginv2::RunStreamRequest>,
    ) -> Result<tonic::Response<Self::RunStreamStream>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConvertFromError::into_tonic_status)?;
        let stream = StreamService::run_stream(self, request)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .map_ok(|packet: StreamPacket<T::JsonValue>| packet.into_plugin_packet())
            .map(|res| match res {
                Ok(x) => Ok(x),
                Err(e) => Err(tonic::Status::internal(e.to_string())),
            });
        Ok(tonic::Response::new(Box::pin(stream)))
    }

    #[tracing::instrument(skip(self), level = "debug")]
    async fn publish_stream(
        &self,
        request: tonic::Request<pluginv2::PublishStreamRequest>,
    ) -> Result<tonic::Response<pluginv2::PublishStreamResponse>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConvertFromError::into_tonic_status)?;
        let response = StreamService::publish_stream(self, request)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .try_into()
            .map_err(|e: serde_json::Error| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(response))
    }
}
