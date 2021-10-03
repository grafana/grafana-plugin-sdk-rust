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
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

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
                x.ref_id,
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
                .into_frame("foo")],
            ))
        })
    }
}

#[derive(Debug, Error)]
#[error("Error streaming data")]
struct StreamError;

#[tonic::async_trait]
impl backend::StreamService for MyPluginService {
    type JsonValue = ();
    async fn subscribe_stream(
        &self,
        request: backend::SubscribeStreamRequest,
    ) -> backend::SubscribeStreamResponse {
        eprintln!("Subscribing to stream");
        let status = if request.path == "stream" {
            backend::SubscribeStreamStatus::Ok
        } else {
            backend::SubscribeStreamStatus::NotFound
        };
        backend::SubscribeStreamResponse {
            status,
            initial_data: None,
        }
    }

    type StreamError = StreamError;
    type Stream = backend::BoxRunStream<Self::StreamError>;
    async fn run_stream(&self, _request: backend::RunStreamRequest) -> Self::Stream {
        eprintln!("Running stream");
        let mut frame = data::Frame::new("foo");
        let initial_data: [u32; 0] = [];
        frame.add_field(initial_data.into_field("x"));
        let mut x = 0u32;
        let n = 3;
        let frame = Arc::new(RwLock::new(frame));
        Box::pin(
            async_stream::stream! {
                loop {
                    let frame = Arc::clone(&frame);
                    if frame.write().await.fields[0]
                        .set_values(x..(x + n))
                        .is_ok()
                    {
                        eprintln!("Yielding frame from {} to {}", x, x+n);
                        x = x + n;
                        yield Ok(backend::StreamPacket::MutableFrame(frame))
                    } else {
                        yield Err(StreamError)
                    }
                }
            }
            .throttle(Duration::from_secs(1)),
        )
    }

    async fn publish_stream(
        &self,
        _request: backend::PublishStreamRequest,
    ) -> backend::PublishStreamResponse {
        eprintln!("Publishing to stream");
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
        .start(listener)
        .await?;

    Ok(())
}
