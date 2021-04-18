use std::{convert::TryFrom, time::Duration};

use chrono::prelude::*;
use serde_json::Value;
use thiserror::Error;

use crate::{data, pluginv2};

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("time_range missing from query")]
    MissingTimeRange,
    #[error("invalid JSON in query (got {json})")]
    InvalidJson {
        err: serde_json::Error,
        json: String,
    },
}

type ConversionResult<T> = std::result::Result<T, ConversionError>;

pub struct DataQuery {
    /// The unique identifier of the query, set by the frontend call.
    pub ref_id: String,

    /// An identifier for the type of query.
    ///
    /// This can be used to distinguish different types of queries.
    query_type: String,

    /// The maximum number of datapoints that should be returned from a time series query.
    max_data_points: i64,

    /// The suggested duration between time points in a time series query.
    interval: Duration,

    /// The start and end of the query requested by the frontend.
    time_range: TimeRange,

    /// The raw JSON query.
    ///
    /// This contains all of the other properties, as well as custom properties.
    json: Value,
}

impl TryFrom<pluginv2::DataQuery> for DataQuery {
    type Error = ConversionError;
    fn try_from(mut other: pluginv2::DataQuery) -> ConversionResult<Self> {
        let json = std::mem::take(&mut other.json);
        Ok(Self {
            ref_id: other.ref_id,
            query_type: other.query_type,
            max_data_points: other.max_data_points,
            interval: Duration::from_millis(other.interval_ms as u64),
            time_range: other
                .time_range
                .map(TimeRange::from)
                .ok_or(ConversionError::MissingTimeRange)?,
            json: serde_json::from_slice(&json).map_err(|err| ConversionError::InvalidJson {
                err,
                json: String::from_utf8(json.clone()).unwrap_or_else(|_| {
                    format!("non-utf8 string: {}", String::from_utf8_lossy(&json))
                }),
            })?,
        })
    }
}

/// The time range for a query.
pub struct TimeRange {
    /// The start time of the query.
    pub from: DateTime<Utc>,
    /// The end time of the query.
    pub to: DateTime<Utc>,
}

impl From<pluginv2::TimeRange> for TimeRange {
    fn from(other: pluginv2::TimeRange) -> Self {
        Self {
            from: Utc.timestamp_millis(other.from_epoch_ms),
            to: Utc.timestamp_millis(other.to_epoch_ms),
        }
    }
}

/// The results from a `DataQuery`.
pub struct DataResponse {
    /// The unique identifier of the query, set by the frontend call.
    ref_id: String,

    /// The data returned from the query.
    frames: Vec<data::Frame>,
}

impl DataResponse {
    pub fn new(ref_id: String, frames: Vec<data::Frame>) -> Self {
        Self { ref_id, frames }
    }
}

pub trait DataQueryError: std::error::Error {
    fn ref_id(self) -> String;
}

#[tonic::async_trait]
pub trait DataService {
    type QueryError: DataQueryError;
    async fn query_data(
        &self,
        queries: Vec<DataQuery>,
    ) -> Vec<Result<DataResponse, Self::QueryError>>;
}

#[tonic::async_trait]
impl<T> pluginv2::data_server::Data for T
where
    T: DataService + Send + Sync + 'static,
{
    async fn query_data(
        &self,
        request: tonic::Request<pluginv2::QueryDataRequest>,
    ) -> Result<tonic::Response<pluginv2::QueryDataResponse>, tonic::Status> {
        let queries = request
            .into_inner()
            .queries
            .into_iter()
            .map(DataQuery::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        let responses = DataService::query_data(self, queries)
            .await
            .into_iter()
            .map(|resp| match resp {
                Ok(x) => (
                    x.ref_id.clone(),
                    pluginv2::DataResponse {
                        frames: data::to_arrow(x.frames, x.ref_id),
                        error: "".to_string(),
                        json_meta: vec![],
                    },
                ),
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
            .collect();
        Ok(tonic::Response::new(pluginv2::QueryDataResponse {
            responses,
        }))
    }
}
