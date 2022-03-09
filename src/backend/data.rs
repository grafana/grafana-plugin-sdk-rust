//! SDK types and traits relevant to plugins that query data.
use std::{collections::HashMap, pin::Pin, time::Duration};

use futures_core::Stream;
use futures_util::StreamExt;
use serde_json::Value;

use crate::{
    backend::{self, ConvertFromError, TimeRange},
    data, pluginv2,
};

/// A request for data made by Grafana.
///
/// Details of the request source can be found in `plugin_context`,
/// while the actual plugins themselves are in `queries`.
#[derive(Debug)]
#[non_exhaustive]
pub struct QueryDataRequest {
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
    pub queries: Vec<DataQuery>,
}

impl TryFrom<pluginv2::QueryDataRequest> for QueryDataRequest {
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::QueryDataRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_context: other
                .plugin_context
                .ok_or(ConvertFromError::MissingPluginContext)
                .and_then(TryInto::try_into)?,
            headers: other.headers,
            queries: other
                .queries
                .into_iter()
                .map(DataQuery::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// A query made by Grafana to the plugin as part of a [`QueryDataRequest`].
///
/// The `json` field contains any fields set by the plugin's UI.
#[derive(Debug)]
#[non_exhaustive]
pub struct DataQuery {
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

    /// The raw JSON query.
    ///
    /// This contains all of the other properties, as well as custom properties.
    pub json: Value,
}

impl TryFrom<pluginv2::DataQuery> for DataQuery {
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
            json: backend::read_json(&other.json)?,
        })
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
    frames: Result<Vec<Vec<u8>>, data::Error>,
}

impl DataResponse {
    /// Create a new [`DataResponse`] with the given `ref_id` and `frames`.
    #[must_use]
    pub fn new(ref_id: String, frames: Vec<data::CheckedFrame<'_>>) -> Self {
        Self {
            ref_id: ref_id.clone(),
            frames: to_arrow(frames, &Some(ref_id)),
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
///     async fn query_data(&self, request: backend::QueryDataRequest) -> Self::Stream {
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
    /// The error type that can be returned by individual queries.
    ///
    /// This must implement [`DataQueryError`], which allows the SDK to
    /// align queries up with any failed requests.
    type QueryError: DataQueryError;

    /// The type of stream returned by the `query_data` method.
    ///
    /// This will generally be impossible to name directly, so returning the
    /// [`BoxDataResponseStream`] type alias will probably be more convenient.
    type Stream<'a>: Stream<Item = Result<DataResponse, Self::QueryError>> + Send + 'a
    where
        Self: 'a;

    /// Query data for an input request.
    ///
    /// The request will contain zero or more queries, as well as information about the
    /// origin of the queries (such as the datasource instance) in the `plugin_context` field.
    async fn query_data(&self, request: QueryDataRequest) -> Self::Stream<'_>;
}

/// Type alias for a boxed iterator of query responses, useful for returning from [`DataService::query_data`].
pub type BoxDataResponseStream<'a, E> =
    Pin<Box<dyn Stream<Item = Result<backend::DataResponse, E>> + Send + 'a>>;

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
                .into_inner()
                .try_into()
                .map_err(ConvertFromError::into_tonic_status)?,
        )
        .await
        .map(|resp| match resp {
            Ok(x) => {
                let ref_id = x.ref_id;
                x.frames.map_or_else(
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
