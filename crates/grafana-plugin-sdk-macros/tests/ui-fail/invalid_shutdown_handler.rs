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

    #[grafana_plugin_sdk::main(services(data), shutdown_handler = true)]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}

mod b {
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

    #[grafana_plugin_sdk::main(services(data), shutdown_handler("127.0.0.1:10001"))]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}

fn main() {}
