//! SDK types and traits relevant to plugins that query data.
use std::{collections::HashMap, pin::Pin, time::Duration};

use bytes::Bytes;
use futures_core::Stream;
use futures_util::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use tonic::transport::Endpoint;

use crate::{
    backend::{self, ConvertFromError, TimeRange},
    data, pluginv2,
};

use super::ConvertToError;

/// Transport metadata sent alongside a query.
///
/// This should be passed into [`DataClient::query_data`] when forwarding
/// a query to the same Grafana instance as the query originated.
#[derive(Clone, Debug)]
pub struct TransportMetadata(tonic::metadata::MetadataMap);

/// A request for data made by Grafana.
///
/// Details of the request source can be found in `plugin_context`,
/// while the actual plugins themselves are in `queries`.
#[derive(Debug)]
#[non_exhaustive]
pub struct QueryDataRequest<Q> {
    /// Details of the plugin instance from which the request originated.
    ///
    /// If the request originates from a datasource instance, this will
    /// include details about the datasource instance in the
    /// `data_source_instance_settings` field.
    pub plugin_context: backend::PluginContext,
    /// Headers included along with the request by Grafana.
    pub headers: HashMap<String, String>,
    /// The queries requested by a user or alert.
    ///
    /// Each [`DataQuery`] contains a unique `ref_id` field which identifies
    /// the query to the frontend; this should be included in the corresponding
    /// `DataResponse` for each query.
    pub queries: Vec<DataQuery<Q>>,

    grpc_meta: TransportMetadata,
}

impl<Q> QueryDataRequest<Q> {
    /// Get the transport metadata in this request.
    ///
    /// This is required when using the [`DataClient`] to query
    /// other Grafana datasources.
    pub fn transport_metadata(&self) -> &TransportMetadata {
        &self.grpc_meta
    }
}

impl<Q> TryFrom<tonic::Request<pluginv2::QueryDataRequest>> for QueryDataRequest<Q>
where
    Q: DeserializeOwned,
{
    type Error = ConvertFromError;
    fn try_from(other: tonic::Request<pluginv2::QueryDataRequest>) -> Result<Self, Self::Error> {
        // Clone is required until https://github.com/hyperium/tonic/pull/1118 is released.
        let grpc_meta = other.metadata().clone();
        let request = other.into_inner();
        Ok(Self {
            plugin_context: request
                .plugin_context
                .ok_or(ConvertFromError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            headers: request.headers,
            queries: request
                .queries
                .into_iter()
                .map(DataQuery::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            grpc_meta: TransportMetadata(grpc_meta),
        })
    }
}

/// A query made by Grafana to the plugin as part of a [`QueryDataRequest`].
///
/// The `query` field contains any fields set by the plugin's UI.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct DataQuery<Q> {
    /// The unique identifier of the query, set by the frontend call.
    ///
    /// This should be included in the corresponding [`DataResponse`].
    pub ref_id: String,

    /// An identifier for the type of query.
    ///
    /// This can be used to distinguish different types of queries.
    pub query_type: String,

    /// The maximum number of datapoints that should be returned from a time series query.
    pub max_data_points: i64,

    /// The suggested duration between time points in a time series query.
    pub interval: Duration,

    /// The start and end of the query requested by the frontend.
    pub time_range: TimeRange,

    /// The inner query.
    ///
    /// This contains all of the other properties, as well as custom properties.
    pub query: Q,
}

impl<Q> TryFrom<pluginv2::DataQuery> for DataQuery<Q>
where
    Q: DeserializeOwned,
{
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::DataQuery) -> Result<Self, Self::Error> {
        Ok(Self {
            ref_id: other.ref_id,
            query_type: other.query_type,
            max_data_points: other.max_data_points,
            interval: Duration::from_millis(other.interval_ms as u64),
            time_range: other
                .time_range
                .map(TimeRange::from)
                .ok_or(ConvertFromError::MissingTimeRange)?,
            query: backend::read_json(&other.json)?,
        })
    }
}

impl<Q> TryFrom<DataQuery<Q>> for pluginv2::DataQuery
where
    Q: Serialize,
{
    type Error = ConvertToError;

    fn try_from(other: DataQuery<Q>) -> Result<Self, Self::Error> {
        Ok(Self {
            ref_id: other.ref_id,
            max_data_points: other.max_data_points,
            interval_ms: other.interval.as_millis() as i64,
            time_range: Some(other.time_range.into()),
            json: serde_json::to_vec(&other.query)
                .map_err(|err| ConvertToError::InvalidJson { err })?,
            query_type: other.query_type,
        })
    }
}

#[derive(Debug)]
enum DataResponseFrames {
    Deserialized(Vec<data::Frame>),
    Serialized(Result<Vec<Vec<u8>>, data::Error>),
}

impl DataResponseFrames {
    fn into_serialized(self) -> Result<Vec<Vec<u8>>, data::Error> {
        match self {
            Self::Deserialized(frames) => to_arrow(
                frames
                    .iter()
                    .map(|x| x.check())
                    .collect::<Result<Vec<_>, _>>()?,
                &None,
            ),
            Self::Serialized(x) => x,
        }
    }
}

/// The results from a [`DataQuery`].
#[derive(Debug)]
pub struct DataResponse {
    /// The unique identifier of the query, set by the frontend call.
    ///
    /// This is used to align queries in the request to data in the response,
    /// and can be obtained from the [`DataQuery`].
    ref_id: String,

    /// The data returned from the query.
    frames: DataResponseFrames,
}

impl DataResponse {
    /// Create a new [`DataResponse`] with the given `ref_id` and `frames`.
    #[must_use]
    pub fn new(ref_id: String, frames: Vec<data::CheckedFrame<'_>>) -> Self {
        Self {
            ref_id: ref_id.clone(),
            frames: DataResponseFrames::Serialized(to_arrow(frames, &Some(ref_id))),
        }
    }
}

/// Error supertrait used in [`DataService::query_data`].
pub trait DataQueryError: std::error::Error {
    /// Return the `ref_id` of the incoming query to which this error corresponds.
    ///
    /// This allows the SDK to align queries up with any failed requests.
    fn ref_id(self) -> String;
}

/// Used to respond for requests for data from datasources and app plugins.
///
/// Datasource plugins will usually want to implement this trait to perform the
/// bulk of their processing.
///
/// # Example
///
/// ```rust
/// use futures_util::stream::FuturesOrdered;
/// use grafana_plugin_sdk::{backend, data, prelude::*};
/// use thiserror::Error;
///
/// struct MyPlugin;
///
/// /// An error that may occur during a query.
/// ///
/// /// This must store the `ref_id` of the query so that Grafana can line it up.
/// #[derive(Debug, Error)]
/// #[error("Error querying backend for query {ref_id}: {source}")]
/// struct QueryError {
///     source: data::Error,
///     ref_id: String,
/// }
///
/// impl backend::DataQueryError for QueryError {
///     fn ref_id(self) -> String {
///         self.ref_id
///     }
/// }
///
/// #[backend::async_trait]
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
///                             source,
///                         })?],
///                     ))
///                 })
///                 .collect::<FuturesOrdered<_>>(),
///         )
///     }
/// }
/// ```
#[tonic::async_trait]
pub trait DataService {
    /// The type of the JSON query sent from Grafana to the plugin.
    type Query: DeserializeOwned + Send + Sync;

    /// The error type that can be returned by individual queries.
    ///
    /// This must implement [`DataQueryError`], which allows the SDK to
    /// align queries up with any failed requests.
    type QueryError: DataQueryError;

    /// The type of stream returned by the `query_data` method.
    ///
    /// This will generally be impossible to name directly, so returning the
    /// [`BoxDataResponseStream`] type alias will probably be more convenient.
    type Stream: Stream<Item = Result<DataResponse, Self::QueryError>> + Send;

    /// Query data for an input request.
    ///
    /// The request will contain zero or more queries, as well as information about the
    /// origin of the queries (such as the datasource instance) in the `plugin_context` field.
    async fn query_data(&self, request: QueryDataRequest<Self::Query>) -> Self::Stream;
}

/// Type alias for a boxed iterator of query responses, useful for returning from [`DataService::query_data`].
pub type BoxDataResponseStream<E> =
    Pin<Box<dyn Stream<Item = Result<backend::DataResponse, E>> + Send>>;

/// Serialize a slice of frames to Arrow IPC format.
///
/// If `ref_id` is provided, it is passed down to the various conversion
/// function and takes precedence over any `ref_id`s set on the individual frames.
pub(crate) fn to_arrow<'a>(
    frames: impl IntoIterator<Item = data::CheckedFrame<'a>>,
    ref_id: &Option<String>,
) -> Result<Vec<Vec<u8>>, data::Error> {
    frames
        .into_iter()
        .map(|frame| Ok(frame.to_arrow(ref_id.clone())?))
        .collect()
}

#[tonic::async_trait]
impl<T> pluginv2::data_server::Data for T
where
    T: DataService + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self), level = "debug")]
    async fn query_data(
        &self,
        request: tonic::Request<pluginv2::QueryDataRequest>,
    ) -> Result<tonic::Response<pluginv2::QueryDataResponse>, tonic::Status> {
        let responses = DataService::query_data(
            self,
            request
                // .into_inner()
                .try_into()
                .map_err(ConvertFromError::into_tonic_status)?,
        )
        .await
        .map(|resp| match resp {
            Ok(x) => {
                let ref_id = x.ref_id;
                x.frames.into_serialized().map_or_else(
                    |e| {
                        (
                            ref_id.clone(),
                            pluginv2::DataResponse {
                                frames: vec![],
                                error: e.to_string(),
                                json_meta: vec![],
                            },
                        )
                    },
                    |frames| {
                        (
                            ref_id.clone(),
                            pluginv2::DataResponse {
                                frames,
                                error: "".to_string(),
                                json_meta: vec![],
                            },
                        )
                    },
                )
            }
            Err(e) => {
                let err_string = e.to_string();
                (
                    e.ref_id(),
                    pluginv2::DataResponse {
                        frames: vec![],
                        error: err_string,
                        json_meta: vec![],
                    },
                )
            }
        })
        .collect()
        .await;
        Ok(tonic::Response::new(pluginv2::QueryDataResponse {
            responses,
        }))
    }
}

/// A client for querying data from Grafana.
///
/// This can be used by plugins which need to query data from
/// a different datasource of the same Grafana instance.
#[derive(Debug, Clone)]
pub struct DataClient<T = tonic::transport::Channel> {
    inner: pluginv2::data_client::DataClient<T>,
}

impl DataClient<tonic::transport::Channel> {
    /// Create a new DataClient to connect to the given endpoint.
    ///
    /// This constructor uses the default [`tonic::transport::Channel`] as the
    /// transport. Use [`DataClient::with_channel`] to provide your own channel.
    pub fn new(url: impl Into<Bytes>) -> Result<Self, tonic::transport::Error> {
        let endpoint = Endpoint::from_shared(url)?;
        let channel = endpoint.connect_lazy();
        Ok(Self {
            inner: pluginv2::data_client::DataClient::new(channel),
        })
    }
}

/// Errors which can occur when querying data.
#[derive(Clone, Debug, thiserror::Error)]
#[error("Error querying data")]
pub struct QueryDataError;

impl<T> DataClient<T> {
    /// Query for data from a Grafana datasource.
    pub async fn query_data<Q>(
        &mut self,
        queries: Vec<DataQuery<Q>>,
        upstream_metadata: &TransportMetadata,
    ) -> Result<HashMap<String, DataResponse>, QueryDataError>
    where
        Q: Serialize,
        T: tonic::client::GrpcService<tonic::body::BoxBody> + Clone + Send + Sync + 'static,
        T::ResponseBody: tonic::codegen::Body<Data = tonic::codegen::Bytes> + Send + Sync + 'static,
        <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
    {
        let queries: Vec<_> = queries
            .into_iter()
            // TODO: add enum variant
            .map(|q| q.try_into().map_err(|_| QueryDataError))
            .collect::<Result<_, _>>()?;
        let query_data_request = pluginv2::QueryDataRequest {
            plugin_context: None,
            headers: Default::default(),
            queries,
        };
        let mut request = tonic::Request::new(query_data_request);
        for kv in upstream_metadata.0.iter() {
            match kv {
                tonic::metadata::KeyAndValueRef::Ascii(k, v) => {
                    request.metadata_mut().insert(k, v.clone());
                }
                tonic::metadata::KeyAndValueRef::Binary(k, v) => {
                    request.metadata_mut().insert_bin(k, v.clone());
                }
            }
        }
        let responses = self
            .inner
            .query_data(request)
            .await
            // TODO: add enum variant
            .map_err(|_| QueryDataError)?
            .into_inner()
            .responses
            .into_iter()
            .map(|(k, v)| {
                let frames = v
                    .frames
                    .into_iter()
                    .map(|f| data::Frame::from_arrow(f).map_err(|_| QueryDataError))
                    .collect::<Result<Vec<_>, _>>()
                    .map(DataResponseFrames::Deserialized)
                    .unwrap();
                Ok((k.clone(), DataResponse { ref_id: k, frames }))
            })
            .collect::<Result<_, _>>()?;
        Ok(responses)
    }
}
