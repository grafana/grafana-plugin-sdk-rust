//! SDK types and traits relevant to plugins that stream data.
use std::{
    convert::{TryFrom, TryInto},
    pin::Pin,
};

use futures_util::{Stream, StreamExt, TryStreamExt};
use serde::Serialize;

use crate::{
    backend::{ConversionError, PluginContext},
    data, pluginv2,
};

/// A request to subscribe to a stream.
#[derive(Debug)]
pub struct SubscribeStreamRequest {
    /// Details of the plugin instance from which the request originated.
    ///
    /// If the request originates from a datasource instance, this will
    /// include details about the datasource instance in the
    /// `data_source_instance_settings` field.
    pub plugin_context: PluginContext,

    /// The subscription channel path that the request wishes to subscribe to.
    pub path: String,
}

impl TryFrom<pluginv2::SubscribeStreamRequest> for SubscribeStreamRequest {
    type Error = ConversionError;
    fn try_from(other: pluginv2::SubscribeStreamRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConversionError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            path: other.path,
        })
    }
}

/// The status of a subscribe stream response.
#[derive(Clone, Copy, Debug)]
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
pub enum InitialData {
    /// Return a [`Frame`][data::Frame] containing initial data.
    ///
    /// This MUST have the same schema as the data returned in subsequent requests to run the stream.
    Frame(data::Frame<data::Checked>, data::FrameInclude),
    /// Return some arbitrary JSON on stream subscription.
    Json(serde_json::Value),
}

impl TryInto<Vec<u8>> for InitialData {
    type Error = ConversionError;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self {
            Self::Json(v) => {
                Ok(serde_json::to_vec(&v).expect("serde_json::Value contained invalid data"))
            }
            Self::Frame(frame, include) => frame
                .to_json(include)
                .map_err(|source| ConversionError::InvalidFrame { source }),
        }
    }
}

/// The response to a stream subscription request.
///
/// This includes a status and some optional initial data for the stream.
///
/// If `initial_data` is provided then the requirements in the [`InitialData`] documentation
/// MUST be upheld.
#[derive(Debug)]
pub struct SubscribeStreamResponse {
    /// The status of the response.
    pub status: SubscribeStreamStatus,
    /// Optional initial data to return to the client, used to pre-populate the stream.
    pub initial_data: Option<InitialData>,
}

impl TryInto<pluginv2::SubscribeStreamResponse> for SubscribeStreamResponse {
    type Error = ConversionError;
    fn try_into(self) -> Result<pluginv2::SubscribeStreamResponse, Self::Error> {
        let mut response = pluginv2::SubscribeStreamResponse {
            status: 0,
            data: self
                .initial_data
                .map_or_else(|| Ok(vec![]), TryInto::try_into)?,
        };
        response.set_status(self.status.into());
        Ok(response)
    }
}

/// A request to 'run' a stream, i.e. begin streaming data.
///
/// This is made by Grafana _after_ a stream subscription request has been accepted,
/// and will include the same `path` as the subscription request.
#[derive(Debug)]
pub struct RunStreamRequest {
    /// Metadata about the plugin from which the request originated.
    pub plugin_context: PluginContext,
    /// The subscription path; see module level comments for details.
    pub path: String,
}

impl TryFrom<pluginv2::RunStreamRequest> for RunStreamRequest {
    type Error = ConversionError;
    fn try_from(other: pluginv2::RunStreamRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConversionError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            path: other.path,
        })
    }
}

/// A packet of data to be streamed back to the subscribed client.
///
/// Such data can be:
/// - a [`Frame`][data::Frame], which will be serialized to JSON before being sent back to the client
/// - arbitrary JSON
/// - arbitrary bytes.
///
/// The `T` type parameter on this enum is only relevant when JSON data
/// is being streamed back,
#[derive(Debug)]
pub enum StreamPacket<T = ()>
where
    T: Serialize,
{
    /// An owned [`Frame`][data::Frame].
    Frame(data::Frame<data::Checked>),
    /// JSON data of type `T`.
    Json(T),
}

impl<T> StreamPacket<T>
where
    T: Serialize,
{
    fn into_plugin_packet(self) -> Result<pluginv2::StreamPacket, serde_json::Error> {
        Ok(pluginv2::StreamPacket {
            data: match self {
                Self::Frame(f) => f.to_json(data::FrameInclude::All)?,
                Self::Json(j) => serde_json::to_vec(&j)?,
            },
        })
    }
}

/// Type alias for a pinned, boxed stream of stream packets with a custom error type.
pub type BoxRunStream<E, T = ()> =
    Pin<Box<dyn Stream<Item = Result<StreamPacket<T>, E>> + Send + Sync + 'static>>;

/// A request to publish data to a stream.
pub struct PublishStreamRequest {
    /// Details of the plugin instance from which the request originated.
    ///
    /// If the request originates from a datasource instance, this will
    /// include details about the datasource instance in the
    /// `data_source_instance_settings` field.
    pub plugin_context: PluginContext,
    /// The subscription path; see module level comments for details.
    pub path: String,
    /// Data to be published to the stream.
    pub data: serde_json::Value,
}

impl TryFrom<pluginv2::PublishStreamRequest> for PublishStreamRequest {
    type Error = ConversionError;
    fn try_from(other: pluginv2::PublishStreamRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConversionError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            path: other.path,
            data: super::read_json(&other.data)?,
        })
    }
}

/// The status of a publish stream response.
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
pub struct PublishStreamResponse {
    /// The status of the response.
    pub status: PublishStreamStatus,
    /// Data returned in response to publishing.
    pub data: serde_json::Value,
}

impl TryInto<pluginv2::PublishStreamResponse> for PublishStreamResponse {
    type Error = serde_json::Error;
    fn try_into(self) -> Result<pluginv2::PublishStreamResponse, Self::Error> {
        let mut response = pluginv2::PublishStreamResponse {
            status: 0,
            data: serde_json::to_vec(&self.data)?,
        };
        response.set_status(self.status.into());
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
/// struct MyPlugin;
///
/// #[derive(Debug, Error)]
/// #[error("Error streaming data")]
/// struct StreamError;
///
/// impl From<data::FrameError> for StreamError {
///     fn from(_other: data::FrameError) -> StreamError {
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
///         request: backend::SubscribeStreamRequest,
///     ) -> backend::SubscribeStreamResponse {
///         let status = if request.path == "stream" {
///             backend::SubscribeStreamStatus::Ok
///         } else {
///             backend::SubscribeStreamStatus::NotFound
///         };
///         info!(path = %request.path, "Subscribing to stream");
///         backend::SubscribeStreamResponse {
///             status,
///             initial_data: None,
///         }
///     }
///
///     type StreamError = StreamError;
///     type Stream = backend::BoxRunStream<Self::StreamError>;
///
///     /// Begin streaming data for a request.
///     ///
///     /// This example just creates an in-memory `Frame` in each loop iteration,
///     /// sends an updated version of the frame once per second, and updates a loop variable
///     /// so that each frame is different.
///     async fn run_stream(&self, _request: backend::RunStreamRequest) -> Self::Stream {
///         info!("Running stream");
///         let mut x = 0u32;
///         let n = 3;
///         Box::pin(
///             async_stream::try_stream! {
///                 loop {
///                     let frame = data::Frame::from_fields("foo", [
///                         (x..x+n).into_field("x"),
///                     ]).check()?;
///                     eprintln!("Yielding frame from {} to {}", x, x+n);
///                     yield backend::StreamPacket::Frame(frame);
///                     x += n;
///                 }
///             }
///             .throttle(Duration::from_secs(1)),
///         )
///     }
///
///     /// Handle a request to publish data to a stream.
///     ///
///     /// Currently unimplemented in this example, but the functionality _should_ work.
///     async fn publish_stream(
///         &self,
///         _request: backend::PublishStreamRequest,
///     ) -> backend::PublishStreamResponse {
///         info!("Publishing to stream");
///         todo!()
///     }
/// }
/// ```
#[tonic::async_trait]
pub trait StreamService {
    /// Handle requests to begin a subscription to a plugin or datasource managed channel path.
    ///
    /// Implementations should check the subscribe permissions of the incoming request, and can
    /// choose to return some initial data to prepopulate the stream.
    ///
    /// `run_stream` will be called shortly after returning a response with [`SubscribeStreamStatus::Ok`].
    async fn subscribe_stream(&self, request: SubscribeStreamRequest) -> SubscribeStreamResponse;

    /// The type of JSON values returned by this stream service.
    ///
    /// Each [`StreamPacket`] can return either a [`data::Frame`] or some arbitary JSON. This
    /// associated type allows the JSON value to be statically typed, if desired.
    ///
    /// If the implementation does not intend to return any [`StreamPacket::Json`] variants, this
    /// can be set to `()`. If the structure of the returned JSON is not statically known, this
    /// should be set to [`serde_json::Value`].
    type JsonValue: Serialize + Send + Sync;

    /// The type of error that can occur while fetching a stream packet.
    type StreamError: std::error::Error + Send + Sync;

    /// The type of stream returned by `run_stream`.
    ///
    /// This will generally be impossible to name directly, so returning the
    /// [`BoxRunStream`] type alias will probably be more convenient.
    type Stream: futures_core::Stream<Item = Result<StreamPacket<Self::JsonValue>, Self::StreamError>>
        + Send
        + Sync
        + 'static;

    /// Begin sending stream packets to a client.
    ///
    /// This will be called by Grafana after a successful subscription to a channel path.
    ///
    /// When Grafana detects that there are no longer any subscribers to a channel, the stream
    /// will be terminated until the next active subscriber appears. Stream termination can
    /// may be slightly delayed.
    async fn run_stream(&self, request: RunStreamRequest) -> Self::Stream;

    /// Handle requests to publish to a plugin or datasource managed channel path (currently unimplemented).
    ///
    /// Implementations should check the publish permissions of the incoming request.
    async fn publish_stream(&self, request: PublishStreamRequest) -> PublishStreamResponse;
}

#[tonic::async_trait]
impl<T> pluginv2::stream_server::Stream for T
where
    T: std::fmt::Debug + Send + Sync + StreamService + 'static,
{
    #[tracing::instrument(level = "debug")]
    async fn subscribe_stream(
        &self,
        request: tonic::Request<pluginv2::SubscribeStreamRequest>,
    ) -> Result<tonic::Response<pluginv2::SubscribeStreamResponse>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConversionError::into_tonic_status)?;
        let response = StreamService::subscribe_stream(self, request)
            .await
            .try_into()
            .map_err(ConversionError::into_tonic_status)?;
        Ok(tonic::Response::new(response))
    }

    type RunStreamStream = Pin<
        Box<
            dyn futures_core::Stream<Item = Result<pluginv2::StreamPacket, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    #[tracing::instrument(level = "debug")]
    async fn run_stream(
        &self,
        request: tonic::Request<pluginv2::RunStreamRequest>,
    ) -> Result<tonic::Response<Self::RunStreamStream>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConversionError::into_tonic_status)?;
        let stream = StreamService::run_stream(self, request)
            .await
            .map_ok(|packet: StreamPacket<T::JsonValue>| packet.into_plugin_packet())
            .map(|res| match res {
                Ok(Ok(x)) => Ok(x),
                Ok(Err(e)) => Err(tonic::Status::internal(e.to_string())),
                Err(e) => Err(tonic::Status::internal(e.to_string())),
            });
        Ok(tonic::Response::new(Box::pin(stream)))
    }

    #[tracing::instrument(level = "debug")]
    async fn publish_stream(
        &self,
        request: tonic::Request<pluginv2::PublishStreamRequest>,
    ) -> Result<tonic::Response<pluginv2::PublishStreamResponse>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConversionError::into_tonic_status)?;
        let response = StreamService::publish_stream(self, request)
            .await
            .try_into()
            .map_err(|e: serde_json::Error| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(response))
    }
}
