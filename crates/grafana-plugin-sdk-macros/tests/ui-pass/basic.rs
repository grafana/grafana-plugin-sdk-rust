#![allow(dead_code, unused_variables)]

mod a {
    use grafana_plugin_sdk::{backend, data, prelude::*};
    use serde::Deserialize;

    #[derive(Clone, GrafanaPlugin)]
    #[grafana_plugin(plugin_type = "datasource")]
    struct MyPlugin;

    #[derive(Debug, Deserialize)]
    struct Query {
        pub expression: String,
        pub other_user_input: u64,
    }

    #[derive(Debug)]
    struct QueryError {
        source: data::Error,
        ref_id: String,
    }

    impl std::fmt::Display for QueryError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Error in query {}: {}", self.ref_id, self.source)
        }
    }

    impl std::error::Error for QueryError {}

    impl backend::DataQueryError for QueryError {
        fn ref_id(self) -> String {
            self.ref_id
        }
    }

    #[backend::async_trait]
    impl backend::DataService for MyPlugin {
        type Query = Query;
        type QueryError = QueryError;
        type Stream = backend::BoxDataResponseStream<Self::QueryError>;
        async fn query_data(
            &self,
            request: backend::QueryDataRequest<Self::Query, Self>,
        ) -> Self::Stream {
            todo!()
        }
    }

    #[grafana_plugin_sdk::main(services(data))]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}

mod b {
    use std::sync::Arc;

    use grafana_plugin_sdk::{backend, data, prelude::*};
    use http::Response;
    use serde::Deserialize;
    use thiserror::Error;

    #[derive(Clone, GrafanaPlugin)]
    #[grafana_plugin(plugin_type = "datasource")]
    struct MyPlugin;

    #[derive(Debug, Deserialize)]
    struct Query {
        pub expression: String,
        pub other_user_input: u64,
    }

    #[derive(Debug)]
    struct QueryError {
        source: data::Error,
        ref_id: String,
    }

    impl std::fmt::Display for QueryError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Error in query {}: {}", self.ref_id, self.source)
        }
    }

    impl std::error::Error for QueryError {}

    impl backend::DataQueryError for QueryError {
        fn ref_id(self) -> String {
            self.ref_id
        }
    }

    #[backend::async_trait]
    impl backend::DataService for MyPlugin {
        type Query = Query;
        type QueryError = QueryError;
        type Stream = backend::BoxDataResponseStream<Self::QueryError>;
        async fn query_data(
            &self,
            request: backend::QueryDataRequest<Self::Query, Self>,
        ) -> Self::Stream {
            todo!()
        }
    }

    #[backend::async_trait]
    impl backend::DiagnosticsService for MyPlugin {
        type CheckHealthError = std::convert::Infallible;

        async fn check_health(
            &self,
            request: backend::CheckHealthRequest<Self>,
        ) -> Result<backend::CheckHealthResponse, Self::CheckHealthError> {
            todo!()
        }

        type CollectMetricsError = Arc<dyn std::error::Error + Send + Sync>;

        async fn collect_metrics(
            &self,
            request: backend::CollectMetricsRequest<Self>,
        ) -> Result<backend::CollectMetricsResponse, Self::CollectMetricsError> {
            todo!()
        }
    }

    #[derive(Debug, Error)]
    enum ResourceError {
        #[error("HTTP error: {0}")]
        Http(#[from] http::Error),

        #[error("Path not found")]
        NotFound,
    }

    impl backend::ErrIntoHttpResponse for ResourceError {}

    #[backend::async_trait]
    impl backend::ResourceService for MyPlugin {
        type Error = ResourceError;
        type InitialResponse = Response<Vec<u8>>;
        type Stream = backend::BoxResourceStream<Self::Error>;
        async fn call_resource(
            &self,
            r: backend::CallResourceRequest<Self>,
        ) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
            todo!()
        }
    }

    #[backend::async_trait]
    impl backend::StreamService for MyPlugin {
        type JsonValue = ();
        async fn subscribe_stream(
            &self,
            request: backend::SubscribeStreamRequest<Self>,
        ) -> Result<backend::SubscribeStreamResponse, Self::Error> {
            todo!()
        }
        type Error = Arc<dyn std::error::Error>;
        type Stream = backend::BoxRunStream<Self::Error>;
        async fn run_stream(
            &self,
            _request: backend::RunStreamRequest<Self>,
        ) -> Result<Self::Stream, Self::Error> {
            todo!()
        }
        async fn publish_stream(
            &self,
            _request: backend::PublishStreamRequest<Self>,
        ) -> Result<backend::PublishStreamResponse, Self::Error> {
            todo!()
        }
    }

    #[grafana_plugin_sdk::main(services(data, diagnostics, resource, stream))]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}

fn main() {}
