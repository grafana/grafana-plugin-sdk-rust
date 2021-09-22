use std::pin::Pin;

use futures::Stream;
use thiserror::Error;
use tonic::{transport::Server, Request, Response, Status};

use grafana_plugin_sdk::{
    backend, data,
    pluginv2::{
        data_server::DataServer,
        resource_server::{Resource, ResourceServer},
        CallResourceRequest, CallResourceResponse,
    },
};

#[derive(Clone, Debug, Default)]
pub struct MyPluginService {}

#[tonic::async_trait]
impl Resource for MyPluginService {
    type CallResourceStream =
        Pin<Box<dyn Stream<Item = Result<CallResourceResponse, Status>> + Send + Sync + 'static>>;

    async fn call_resource(
        &self,
        request: Request<CallResourceRequest>,
    ) -> Result<Response<Self::CallResourceStream>, Status> {
        todo!()
    }
}

#[derive(Debug, Error)]
#[error("Error querying backend for {}", .ref_id)]
pub struct Error {
    ref_id: String,
}

impl backend::DataQueryError for Error {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

#[tonic::async_trait]
impl backend::DataService for MyPluginService {
    type QueryError = Error;
    type Iter = std::iter::Map<
        std::vec::IntoIter<backend::DataQuery>,
        fn(backend::DataQuery) -> Result<backend::DataResponse, Self::QueryError>,
    >;
    async fn query_data(&self, queries: Vec<backend::DataQuery>) -> Self::Iter {
        queries.into_iter().map(|x| {
            Ok(backend::DataResponse::new(
                x.ref_id,
                vec![data::Frame::new("foo".to_string())],
            ))
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse()?;
    println!("1|2|tcp|{}|grpc", addr);

    let plugin = MyPluginService {};

    let resource_svc = ResourceServer::new(plugin.clone());
    let data_svc = DataServer::new(plugin.clone());

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ResourceServer<MyPluginService>>()
        .await;
    health_reporter
        .set_serving::<DataServer<MyPluginService>>()
        .await;

    Server::builder()
        .add_service(health_service)
        .add_service(resource_svc)
        .add_service(data_svc)
        .serve(addr)
        .await?;

    Ok(())
}
