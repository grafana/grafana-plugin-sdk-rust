use std::{pin::Pin, sync::Arc, time::Duration};

use chrono::prelude::*;
use futures::Stream;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tonic::transport::Server;

use grafana_plugin_sdk::{
    backend,
    data::{self, prelude::*},
};

#[derive(Clone, Debug, Default)]
pub struct MyPluginService {}

#[derive(Debug, Error)]
#[error("Error querying backend for {}", .ref_id)]
pub struct QueryError {
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
            // Create a single response Frame for each query.
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
                    .into_field()
                    .name("time".to_string()),
                    [1u32, 2, 3].into_field().name("value".to_string()),
                    ["a", "b", "c"].into_field().name("value".to_string()),
                ]
                .into_frame("foo".to_string())],
            ))
        })
    }
}

#[derive(Debug, Error)]
#[error("Error streaming data")]
pub struct StreamError;

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
    type Stream = Pin<
        Box<
            dyn Stream<Item = Result<backend::StreamPacket, Self::StreamError>>
                + Send
                + Sync
                + 'static,
        >,
    >;
    async fn run_stream(&self, _request: backend::RunStreamRequest) -> Self::Stream {
        eprintln!("Running stream");
        let mut frame = data::Frame::new("foo".to_string());
        let initial_data: [u32; 0] = [];
        frame.add_field(initial_data.into_field().name("x".to_string()));
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The compiled executable is run by Grafana's backend and is expected
    // to behave as a [go-plugin]. The first thing we need to do upon startup
    // is output the eventual address of our gRPC service to stdout, in a
    // format understood by go-plugin. See [this guide on non-Go languages][guide]
    // for more details.
    //
    // [go-plugin]: https://github.com/hashicorp/go-plugin
    // [guide]: https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md
    let addr = backend::initialize().await?;

    let plugin = MyPluginService {};

    let data_svc = backend::DataServer::new(plugin.clone());
    let stream_svc = backend::StreamServer::new(plugin.clone());

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<backend::DataServer<MyPluginService>>()
        .await;
    health_reporter
        .set_serving::<backend::StreamServer<MyPluginService>>()
        .await;

    Server::builder()
        .add_service(health_service)
        .add_service(data_svc)
        .add_service(stream_svc)
        .serve(addr)
        .await?;

    Ok(())
}
