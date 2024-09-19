//! SDK types and traits relevant to plugins that query data.
use std::{collections::HashMap, fmt, pin::Pin, time::Duration};

use futures_core::Stream;
use futures_util::StreamExt;
use serde::de::DeserializeOwned;

use crate::{
    backend::{
        self, error_source::ErrorSource, ConvertFromError, GrafanaPlugin, InstanceSettings,
        PluginType, TimeRange,
    },
    data, pluginv2,
};

/// A request for data made by Grafana.
///
/// Details of the request source can be found in `plugin_context`,
/// while the actual plugins themselves are in `queries`.
#[derive(Debug)]
#[non_exhaustive]
pub struct InnerQueryDataRequest<Q, IS, JsonData, SecureJsonData>
where
    Q: DeserializeOwned,
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
    /// Details of the plugin instance from which the request originated.
    ///
    /// If the request originates from a datasource instance, this will
    /// include details about the datasource instance in the
    /// `data_source_instance_settings` field.
    pub plugin_context: backend::PluginContext<IS, JsonData, SecureJsonData>,
    /// Headers included along with the request by Grafana.
    pub headers: HashMap<String, String>,
    /// The queries requested by a user or alert.
    ///
    /// Each [`DataQuery`] contains a unique `ref_id` field which identifies
    /// the query to the frontend; this should be included in the corresponding
    /// `DataResponse` for each query.
    pub queries: Vec<DataQuery<Q>>,
}

impl<Q, IS, JsonData, SecureJsonData> TryFrom<pluginv2::QueryDataRequest>
    for InnerQueryDataRequest<Q, IS, JsonData, SecureJsonData>
where
    Q: DeserializeOwned,
    JsonData: fmt::Debug + DeserializeOwned,
    SecureJsonData: DeserializeOwned,
    IS: InstanceSettings<JsonData, SecureJsonData>,
{
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

/// A request for data made by Grafana.
///
/// Details of the request source can be found in `plugin_context`,
/// while the actual plugins themselves are in `queries`.
///
/// This is a convenience type alias to hide some of the complexity of
/// the various generics involved.
///
/// The type parameter `Q` is the type of the query: generally you can use
/// `Self::Query` here.
///
/// The type parameter `T` is the type of the plugin implementation itself,
/// which must implement [`GrafanaPlugin`].
pub type QueryDataRequest<Q, T> = InnerQueryDataRequest<
    Q,
    <<T as GrafanaPlugin>::PluginType as PluginType<
        <T as GrafanaPlugin>::JsonData,
        <T as GrafanaPlugin>::SecureJsonData,
    >>::InstanceSettings,
    <T as GrafanaPlugin>::JsonData,
    <T as GrafanaPlugin>::SecureJsonData,
>;

/// A query made by Grafana to the plugin as part of a [`QueryDataRequest`].
///
/// The `query` field contains any fields set by the plugin's UI.
#[derive(Debug)]
#[non_exhaustive]
pub struct DataQuery<Q>
where
    Q: DeserializeOwned,
{
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
            query: backend::read_json_query(&other.json)?,
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

    /// A suitable [`DataQueryStatus`] to represent this error.
    ///
    /// This may be used by clients to decide how this error should
    /// be handled. For example, whether the request should be retried,
    /// treated as a client or server error, etc.
    ///
    /// Defaults to [`DataQueryStatus::Unknown`] if not overridden.
    fn status(&self) -> DataQueryStatus {
        DataQueryStatus::Unknown
    }

    /// The source of the error.
    ///
    /// Defaults to [`ErrorSource::Plugin`] if not overridden.
    fn source(&self) -> ErrorSource {
        ErrorSource::default()
    }
}

/// Status codes for [`DataQueryError`].
///
/// These generally correspond to HTTP status codes, but are not
/// a 1:1 mapping: several variants may map to a single HTTP
/// status code, and not all HTTP status codes are given.
///
/// To return a custom HTTP status code in more advanced scenarios,
/// use [`DataQueryStatus::Custom`] and nest the required
/// [`http::StatusCode`] inside.
pub enum DataQueryStatus {
    /// An error that should be updated to contain an accurate status code,
    /// as none has been provided.
    ///
    /// HTTP status code 500.
    Unknown,

    /// The query was successful.
    ///
    /// HTTP status code 200.
    OK,

    /// The datasource does not recognize the client's authentication,
    /// either because it has not been provided
    /// or is invalid for the operation.
    ///
    /// HTTP status code 401.
    Unauthorized,

    /// The datasource refuses to perform the requested action for the authenticated user.
    /// HTTP status code 403.
    Forbidden,

    /// The datasource does not have any corresponding document to return to the request.
    ///
    /// HTTP status code 404.
    NotFound,

    /// The client is rate limited by the datasource and should back-off before trying again.
    ///
    /// HTTP status code 429.
    TooManyRequests,

    /// The datasource was unable to parse the parameters or payload for the request.
    ///
    /// HTTP status code 400.
    BadRequest,

    /// The datasource was able to parse the payload for the request,
    /// but it failed one or more validation checks.
    ///
    /// HTTP status code 400.
    ValidationFailed,

    /// The datasource acknowledges that there's an error, but that there is
    /// nothing the client can do to fix it.
    ///
    /// HTTP status code 500.
    Internal,

    /// The datasource does not support the requested action.
    /// Typically used during development of new features.
    ///
    /// HTTP status code 501.
    NotImplemented,

    /// The datasource did not complete the request within the required time and aborted the action.
    ///
    /// HTTP status code 504.
    Timeout,

    /// The datasource, while acting as a gateway or proxy, received an invalid response
    /// from the upstream server.
    ///
    /// HTTP status code 502.
    BadGateway,

    /// The datasource encountered another error, best represented by
    /// the inner status code.
    Custom(http::StatusCode),
}

impl DataQueryStatus {
    fn as_http(&self) -> http::StatusCode {
        use http::StatusCode;
        match self {
            DataQueryStatus::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            DataQueryStatus::OK => StatusCode::OK,
            DataQueryStatus::Unauthorized => StatusCode::UNAUTHORIZED,
            DataQueryStatus::Forbidden => StatusCode::FORBIDDEN,
            DataQueryStatus::NotFound => StatusCode::NOT_FOUND,
            DataQueryStatus::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            DataQueryStatus::BadRequest => StatusCode::BAD_REQUEST,
            DataQueryStatus::ValidationFailed => StatusCode::BAD_REQUEST,
            DataQueryStatus::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            DataQueryStatus::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            DataQueryStatus::Timeout => StatusCode::GATEWAY_TIMEOUT,
            DataQueryStatus::BadGateway => StatusCode::BAD_GATEWAY,
            DataQueryStatus::Custom(inner) => *inner,
        }
    }

    fn as_i32(&self) -> i32 {
        self.as_http().as_u16().into()
    }
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
/// #[derive(Clone, Debug, GrafanaPlugin)]
/// #[grafana_plugin(plugin_type = "datasource")]
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
///     async fn query_data(&self, request: backend::QueryDataRequest<Self::Query, Self>) -> Self::Stream {
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
pub trait DataService: GrafanaPlugin {
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
    async fn query_data(&self, request: QueryDataRequest<Self::Query, Self>) -> Self::Stream;
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
                                status: DataQueryStatus::Internal.as_i32(),
                                error: e.to_string(),
                                json_meta: vec![],
                                error_source: ErrorSource::Plugin.to_string(),
                            },
                        )
                    },
                    |frames| {
                        (
                            ref_id.clone(),
                            pluginv2::DataResponse {
                                frames,
                                status: DataQueryStatus::OK.as_i32(),
                                error: "".to_string(),
                                json_meta: vec![],
                                error_source: "".to_string(),
                            },
                        )
                    },
                )
            }
            Err(e) => {
                let status = e.status().as_i32();
                let source = e.source().to_string();
                let err_string = e.to_string();
                (
                    e.ref_id(),
                    pluginv2::DataResponse {
                        frames: vec![],
                        status,
                        error: err_string,
                        json_meta: vec![],
                        error_source: source,
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
