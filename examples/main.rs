use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use bytes::Bytes;
use chrono::prelude::*;
use http::Response;
use thiserror::Error;
use tokio_stream::StreamExt;
use tracing::{debug, info};

use grafana_plugin_sdk::{backend, data, prelude::*};

#[derive(Clone, Debug, Default)]
struct MyPluginService(Arc<AtomicUsize>);

impl MyPluginService {
    fn new() -> Self {
        Self(Arc::new(AtomicUsize::new(0)))
    }
}

// Data service implementation.

#[derive(Debug, Error)]
#[error("Error querying backend for query {ref_id}: {source}")]
struct QueryError {
    source: data::Error,
    ref_id: String,
}

impl backend::DataQueryError for QueryError {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

type QueryDataIter = std::iter::Map<
    std::vec::IntoIter<backend::DataQuery>,
    fn(backend::DataQuery) -> Result<backend::DataResponse, QueryError>,
>;

#[tonic::async_trait]
impl backend::DataService for MyPluginService {
    type QueryError = QueryError;
    type Iter = QueryDataIter;
    async fn query_data(&self, request: backend::QueryDataRequest) -> Self::Iter {
        request.queries.into_iter().map(|x| {
            // Here we create a single response Frame for each query.
            // Frames can be created from iterators of fields using [`IntoFrame`].
            Ok(backend::DataResponse::new(
                x.ref_id.clone(),
                vec![[
                    // Fields can be created from iterators of a variety of
                    // relevant datatypes.
                    [
                        Utc.ymd(2021, 1, 1).and_hms(12, 0, 0),
                        Utc.ymd(2021, 1, 1).and_hms(12, 0, 1),
                        Utc.ymd(2021, 1, 1).and_hms(12, 0, 2),
                    ]
                    .into_field("time"),
                    [1_u32, 2, 3].into_field("x"),
                    ["a", "b", "c"].into_field("y"),
                ]
                .into_frame("foo")
                .check()
                .map_err(|source| QueryError {
                    ref_id: x.ref_id,
                    source,
                })?],
            ))
        })
    }
}

// Stream service implementation.

#[derive(Debug, Error)]
#[error("Error streaming data")]
enum StreamError {
    #[error("Error converting frame: {0}")]
    Conversion(#[from] backend::ConvertToError),
    #[error("Invalid frame returned: {0}")]
    InvalidFrame(#[from] data::Error),
}

#[tonic::async_trait]
impl backend::StreamService for MyPluginService {
    type JsonValue = ();
    async fn subscribe_stream(
        &self,
        request: backend::SubscribeStreamRequest,
    ) -> Result<backend::SubscribeStreamResponse, Self::Error> {
        let status = if request.path == "stream" {
            backend::SubscribeStreamStatus::Ok
        } else {
            backend::SubscribeStreamStatus::NotFound
        };
        info!(path = %request.path, "Subscribing to stream");
        Ok(backend::SubscribeStreamResponse {
            status,
            initial_data: None,
        })
    }

    type Error = StreamError;
    type Stream = backend::BoxRunStream<Self::Error>;
    async fn run_stream(
        &self,
        _request: backend::RunStreamRequest,
    ) -> Result<Self::Stream, Self::Error> {
        info!("Running stream");
        let mut x = 0u32;
        let n = 3;
        let mut frame = data::Frame::new("foo").with_field((x..x + n).into_field("x"));
        Ok(Box::pin(
            async_stream::try_stream! {
                loop {
                    frame.fields_mut()[0].set_values(
                        (x..x+n)
                    )?;
                    let packet = backend::StreamPacket::from_frame(frame.check()?)?;
                    debug!("Yielding frame from {} to {}", x, x+n);
                    yield packet;
                    x += n;
                }
            }
            .throttle(Duration::from_secs(1)),
        ))
    }

    async fn publish_stream(
        &self,
        _request: backend::PublishStreamRequest,
    ) -> Result<backend::PublishStreamResponse, Self::Error> {
        info!("Publishing to stream");
        todo!()
    }
}

// Resource service implementation.

#[derive(Debug, Error)]
enum ResourceError {
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),

    #[error("Not found")]
    NotFound,
}

impl backend::ErrIntoHttpResponse for ResourceError {
    fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
        let status = match &self {
            Self::Http(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => http::StatusCode::NOT_FOUND,
        };
        Ok(Response::builder()
            .status(status)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Bytes::from(serde_json::to_vec(
                &serde_json::json!({"error": self.to_string()}),
            )?))?)
    }
}

#[tonic::async_trait]
impl backend::ResourceService for MyPluginService {
    type Error = ResourceError;
    type InitialResponse = http::Response<Bytes>;
    type Stream = backend::BoxResourceStream<Self::Error>;
    async fn call_resource(
        &self,
        r: backend::CallResourceRequest,
    ) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
        let count = Arc::clone(&self.0);
        let response_and_stream = match r.request.uri().path() {
            // Just send back a single response.
            "/echo" => Ok((
                Response::new(r.request.into_body()),
                Box::pin(futures::stream::empty()) as Self::Stream,
            )),
            // Send an initial response with the current count, then stream the gradually
            // incrementing count back to the client.
            "/count" => Ok((
                Response::new(
                    count
                        .fetch_add(1, Ordering::SeqCst)
                        .to_string()
                        .into_bytes()
                        .into(),
                ),
                Box::pin(async_stream::try_stream! {
                    loop {
                        let body = count
                            .fetch_add(1, Ordering::SeqCst)
                            .to_string()
                            .into_bytes()
                            .into();
                        yield body;
                    }
                }) as Self::Stream,
            )),
            _ => return Err(ResourceError::NotFound),
        };
        response_and_stream
    }
}

#[grafana_plugin_sdk::main(
    services(data, resource, stream),
    init_subscriber = true,
    shutdown_handler = "0.0.0.0:10001"
)]
async fn plugin() -> MyPluginService {
    MyPluginService::new()
}
