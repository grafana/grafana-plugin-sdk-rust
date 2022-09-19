#![allow(dead_code, unused_variables)]

mod a {
    use grafana_plugin_sdk::{backend, data};

    #[derive(Clone)]
    struct MyPlugin;

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
        type QueryError = QueryError;
        type Iter = backend::BoxDataResponseIter<Self::QueryError>;
        async fn query_data(&self, request: backend::QueryDataRequest) -> Self::Iter {
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

    use grafana_plugin_sdk::{backend, data};

    #[derive(Clone)]
    struct MyPlugin;

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
        type QueryError = QueryError;
        type Iter = backend::BoxDataResponseIter<Self::QueryError>;
        async fn query_data(&self, request: backend::QueryDataRequest) -> Self::Iter {
            todo!()
        }
    }

    #[backend::async_trait]
    impl backend::DiagnosticsService for MyPlugin {
        type CheckHealthError = std::convert::Infallible;

        async fn check_health(
            &self,
            request: backend::CheckHealthRequest,
        ) -> Result<backend::CheckHealthResponse, Self::CheckHealthError> {
            todo!()
        }

        type CollectMetricsError = Arc<dyn std::error::Error + Send + Sync>;

        async fn collect_metrics(
            &self,
            request: backend::CollectMetricsRequest,
        ) -> Result<backend::CollectMetricsResponse, Self::CollectMetricsError> {
            todo!()
        }
    }

    #[backend::async_trait]
    impl backend::ResourceService for MyPlugin {
        type Error = Arc<dyn std::error::Error + Send + Sync>;
        type InitialResponse = Vec<u8>;
        type Stream = backend::BoxResourceStream<Self::Error>;
        async fn call_resource(&self, r: backend::CallResourceRequest) -> (Result<Self::InitialResponse, Self::Error>, Self::Stream) {
            todo!()
        }
    }

    #[backend::async_trait]
    impl backend::StreamService for MyPlugin {
        type JsonValue = ();
        async fn subscribe_stream(
            &self,
            request: backend::SubscribeStreamRequest,
        ) -> backend::SubscribeStreamResponse {
            todo!()
        }
        type StreamError = Arc<dyn std::error::Error>;
        type Stream = backend::BoxRunStream<Self::StreamError>;
        async fn run_stream(&self, _request: backend::RunStreamRequest) -> Self::Stream {
            todo!()
        }
        async fn publish_stream(
            &self,
            _request: backend::PublishStreamRequest,
        ) -> backend::PublishStreamResponse {
            todo!()
        }
    }

    #[grafana_plugin_sdk::main(services(data, diagnostics, resource, stream))]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}

fn main() {}
