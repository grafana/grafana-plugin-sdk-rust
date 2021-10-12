use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

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

#[derive(Debug, Error)]
#[error("Error querying backend for {}", .ref_id)]
struct QueryError {
    ref_id: String,
}

impl backend::DataQueryError for QueryError {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

#[tonic::async_trait]
impl backend::DataService for MyPluginService {
    type QueryError = QueryError;
    type Iter = std::iter::Map<
        std::vec::IntoIter<backend::DataQuery>,
        fn(backend::DataQuery) -> Result<backend::DataResponse, Self::QueryError>,
    >;
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
                .into_checked_frame("foo")
                .map_err(|_| QueryError { ref_id: x.ref_id })?],
            ))
        })
    }
}

#[derive(Debug, Error)]
#[error("Error streaming data")]
struct StreamError;

impl From<data::FrameError> for StreamError {
    fn from(_other: data::FrameError) -> StreamError {
        StreamError
    }
}

#[tonic::async_trait]
impl backend::StreamService for MyPluginService {
    type JsonValue = ();
    async fn subscribe_stream(
        &self,
        request: backend::SubscribeStreamRequest,
    ) -> backend::SubscribeStreamResponse {
        let status = if request.path == "stream" {
            backend::SubscribeStreamStatus::Ok
        } else {
            backend::SubscribeStreamStatus::NotFound
        };
        info!(path = %request.path, "Subscribing to stream");
        backend::SubscribeStreamResponse {
            status,
            initial_data: None,
        }
    }

    type StreamError = StreamError;
    type Stream = backend::BoxRunStream<Self::StreamError>;
    async fn run_stream(&self, _request: backend::RunStreamRequest) -> Self::Stream {
        info!("Running stream");
        let mut x = 0u32;
        let n = 3;
        Box::pin(
            async_stream::try_stream! {
                loop {
                    let frame = data::Frame::from_fields("foo", [
                        (x..x+n).into_field("x"),
                    ]).check()?;
                    debug!("Yielding frame from {} to {}", x, x+n);
                    yield backend::StreamPacket::Frame(frame);
                    x += n;
                }
            }
            .throttle(Duration::from_secs(1)),
        )
    }

    async fn publish_stream(
        &self,
        _request: backend::PublishStreamRequest,
    ) -> backend::PublishStreamResponse {
        info!("Publishing to stream");
        todo!()
    }
}

#[tonic::async_trait]
impl backend::ResourceService for MyPluginService {
    type Error = http::Error;
    type Stream = backend::BoxResourceStream<Self::Error>;
    async fn call_resource(&self, r: backend::CallResourceRequest) -> Self::Stream {
        let count = Arc::clone(&self.0);
        Box::pin(async_stream::stream! {
            match r.request.uri().path() {
                "/echo" => {
                    yield Ok(Response::new(r.request.uri().to_string().into_bytes()));
                    yield Ok(Response::new(format!("{:?}", r.request.headers()).into_bytes()));
                    yield Ok(Response::new(r.request.into_body()));
                },
                "/count" => {
                    yield Ok(Response::new(
                        count.fetch_add(1, Ordering::SeqCst)
                        .to_string()
                        .into_bytes()
                    ))
                },
                _ => yield Response::builder().status(404).body(vec![]),
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = backend::initialize().await?;
    let plugin = MyPluginService::new();

    backend::Plugin::new()
        .data_service(plugin.clone())
        .stream_service(plugin)
        .init_subscriber(true)
        .shutdown_handler(([0, 0, 0, 0], 10001).into())
        .start(listener)
        .await?;

    Ok(())
}
