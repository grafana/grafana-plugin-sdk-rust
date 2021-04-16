use std::pin::Pin;

use futures::Stream;

use tonic::{transport::Server, Request, Response, Status};

use grafana_plugin_sdk::pluginv2::{resource_server::{Resource, ResourceServer}, CallResourceRequest, CallResourceResponse};

#[derive(Debug, Default)]
pub struct MyPluginService {}

#[tonic::async_trait]
impl Resource for MyPluginService {
	type CallResourceStream = Pin<Box<dyn Stream<Item = Result<CallResourceResponse, Status>> + Send + Sync + 'static>>;

	async fn call_resource(&self, request: Request<CallResourceRequest>) -> Result<Response<Self::CallResourceStream>, Status> {
		todo!()
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:10000".parse()?;
	println!("Plugin server listening on {}", addr);

	let plugin = MyPluginService {};

	let svc = ResourceServer::new(plugin);

	Server::builder().add_service(svc).serve(addr).await?;

	Ok(())
}
