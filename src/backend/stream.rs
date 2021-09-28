/// SDK types and traits relevant to plugins that stream data.
use std::{
    convert::{TryFrom, TryInto},
    pin::Pin,
    sync::Arc,
};

use futures_util::{FutureExt, StreamExt, TryStreamExt};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::{
    backend::{ConversionError, PluginContext},
    data, pluginv2,
};

/// A request to subscribe to a stream.
#[derive(Debug)]
pub struct SubscribeStreamRequest {
    pub plugin_context: PluginContext,
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
///
/// This can be:
///
/// - a [`Frame`], in which case it MUST have the same schema as the data returned in subsequent request to run a stream,
/// - arbitrary JSON
#[derive(Debug)]
pub enum InitialData {
    Frame(data::Frame, data::FrameInclude),
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
/// If [`initial_data`] is provided then the requirements in the [`InitialData`] documentation
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
/// and will include the same [`path`] as the subscription request.
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
/// - a [`Frame`], which will be serialized to JSON before being sent back to the client
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
    /// An owned [`Frame`].
    ///
    /// This provides the simplest API, but when streaming lots of data
    /// it may be preferable to use the `MutableFrame` variant which allows
    /// the same `Frame` to be reused.
    Frame(data::Frame),
    /// A shared, mutable [`Frame`].
    MutableFrame(Arc<RwLock<data::Frame>>),
    /// JSON data of type `T`.
    Json(T),
}

impl<T> StreamPacket<T>
where
    T: Serialize,
{
    // async fn into_plugin_packet(self) -> pluginv2::StreamPacket {
    async fn into_plugin_packet(self) -> Result<pluginv2::StreamPacket, serde_json::Error> {
        Ok(pluginv2::StreamPacket {
            data: match self {
                Self::Frame(f) => f.to_json(data::FrameInclude::All)?,
                Self::MutableFrame(f) => f.read().await.to_json(data::FrameInclude::All)?,
                Self::Json(j) => serde_json::to_vec(&j)?,
            },
        })
    }
}

// impl<T> TryInto<pluginv2::StreamPacket> for StreamPacket<'_, T>
// where
//     T: Serialize,
// {
//     type Error = ConversionError;
//     fn try_into(self) -> Result<pluginv2::StreamPacket, Self::Error> {
//         Ok(pluginv2::StreamPacket {
//             data: match self {
//                 Self::Frame(f) => f
//                     .to_json(data::FrameInclude::All)
//                     .map_err(|source| ConversionError::InvalidFrame { source })?,
//                 Self::Json(j) => serde_json::to_vec(j).expect("couldn't serialize packet as JSON"),
//             },
//         })
//     }
// }

/// A request to publish data to a stream.
pub struct PublishStreamRequest {
    /// Metadata about the plugin from which the request originated.
    pub plugin_context: PluginContext,
    /// The subscription path; see module level comments for details.
    pub path: String,
    /// Data to be published to the stream.
    pub data: serde_json::Value,
}

pub enum PublishStreamStatus {
    Ok,
    NotFound,
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

pub struct PublishStreamResponse {
    pub status: PublishStreamStatus,
    pub data: serde_json::Value,
}

#[tonic::async_trait]
pub trait StreamService {
    async fn subscribe_stream(&self, request: SubscribeStreamRequest) -> SubscribeStreamResponse;

    type JsonValue: Serialize + Send + Sync;
    type StreamError: std::error::Error + Send + Sync;
    type Stream: futures_core::Stream<Item = Result<StreamPacket<Self::JsonValue>, Self::StreamError>>
        + Send
        + Sync
        + 'static;
    async fn run_stream(&self, request: RunStreamRequest) -> Self::Stream;

    async fn publish_stream(&self, request: PublishStreamRequest) -> PublishStreamResponse;
}

#[tonic::async_trait]
impl<T> pluginv2::stream_server::Stream for T
where
    T: Send + Sync + StreamService + 'static,
{
    async fn subscribe_stream(
        &self,
        request: tonic::Request<pluginv2::SubscribeStreamRequest>,
    ) -> Result<tonic::Response<pluginv2::SubscribeStreamResponse>, tonic::Status> {
        eprintln!("SDK Subscribing to stream; request: {:?}", request);
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConversionError::into_tonic_status)?;
        eprintln!(
            "SDK Subscribing to stream; converted request: {:?}",
            request
        );
        let response = StreamService::subscribe_stream(self, request)
            .await
            .try_into()
            .map_err(ConversionError::into_tonic_status)?;
        eprintln!(
            "SDK Subscribing to stream; converted response: {:?}",
            response
        );
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
            .and_then(|packet: StreamPacket<T::JsonValue>| packet.into_plugin_packet().map(Ok))
            .map(|res| match res {
                Ok(Ok(x)) => Ok(x),
                Ok(Err(e)) => Err(tonic::Status::internal(e.to_string())),
                Err(e) => Err(tonic::Status::internal(e.to_string())),
            });
        Ok(tonic::Response::new(Box::pin(stream)))
    }

    async fn publish_stream(
        &self,
        _request: tonic::Request<pluginv2::PublishStreamRequest>,
    ) -> Result<tonic::Response<pluginv2::PublishStreamResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented(
            "Stream publishing is not yet implemented in the Rust SDK.",
        ))
    }
}
