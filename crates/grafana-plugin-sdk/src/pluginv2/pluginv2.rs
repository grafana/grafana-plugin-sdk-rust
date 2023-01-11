#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppInstanceSettings {
    #[prost(bytes = "vec", tag = "3")]
    pub json_data: ::prost::alloc::vec::Vec<u8>,
    #[prost(map = "string, string", tag = "4")]
    pub decrypted_secure_json_data: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    #[prost(int64, tag = "5")]
    pub last_updated_ms: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataSourceInstanceSettings {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub url: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub user: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub database: ::prost::alloc::string::String,
    #[prost(bool, tag = "6")]
    pub basic_auth_enabled: bool,
    #[prost(string, tag = "7")]
    pub basic_auth_user: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "8")]
    pub json_data: ::prost::alloc::vec::Vec<u8>,
    #[prost(map = "string, string", tag = "9")]
    pub decrypted_secure_json_data: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    #[prost(int64, tag = "10")]
    pub last_updated_ms: i64,
    #[prost(string, tag = "11")]
    pub uid: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct User {
    #[prost(string, tag = "1")]
    pub login: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub email: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub role: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PluginContext {
    /// The Grafana organization id the request originating from.
    #[prost(int64, tag = "1")]
    pub org_id: i64,
    /// The unique identifier of the plugin the request  originating from.
    #[prost(string, tag = "2")]
    pub plugin_id: ::prost::alloc::string::String,
    /// The Grafana user the request originating from.
    ///
    /// Will not be provided if Grafana backend initiated the request.
    #[prost(message, optional, tag = "3")]
    pub user: ::core::option::Option<User>,
    /// App plugin instance settings is the configured app instance settings.
    /// In Grafana an app instance is an enabled app plugin in a
    /// Grafana organization.
    ///
    /// Will only be set if request targeting an app instance.
    #[prost(message, optional, tag = "4")]
    pub app_instance_settings: ::core::option::Option<AppInstanceSettings>,
    /// Data source instance settings is the configured data source instance
    /// settings. In Grafana a data source instance is a created data source
    /// in a Grafana organization.
    ///
    /// Will only be set if request targeting a data source instance.
    #[prost(message, optional, tag = "5")]
    pub data_source_instance_settings: ::core::option::Option<
        DataSourceInstanceSettings,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringList {
    #[prost(string, repeated, tag = "1")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallResourceRequest {
    #[prost(message, optional, tag = "1")]
    pub plugin_context: ::core::option::Option<PluginContext>,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub method: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub url: ::prost::alloc::string::String,
    #[prost(map = "string, message", tag = "5")]
    pub headers: ::std::collections::HashMap<::prost::alloc::string::String, StringList>,
    #[prost(bytes = "bytes", tag = "6")]
    pub body: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallResourceResponse {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(map = "string, message", tag = "2")]
    pub headers: ::std::collections::HashMap<::prost::alloc::string::String, StringList>,
    #[prost(bytes = "bytes", tag = "3")]
    pub body: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeRange {
    #[prost(int64, tag = "1")]
    pub from_epoch_ms: i64,
    #[prost(int64, tag = "2")]
    pub to_epoch_ms: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataQuery {
    #[prost(string, tag = "1")]
    pub ref_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    pub max_data_points: i64,
    #[prost(int64, tag = "3")]
    pub interval_ms: i64,
    #[prost(message, optional, tag = "4")]
    pub time_range: ::core::option::Option<TimeRange>,
    #[prost(bytes = "vec", tag = "5")]
    pub json: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "6")]
    pub query_type: ::prost::alloc::string::String,
}
/// QueryDataRequest
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDataRequest {
    #[prost(message, optional, tag = "1")]
    pub plugin_context: ::core::option::Option<PluginContext>,
    /// Environment info
    #[prost(map = "string, string", tag = "2")]
    pub headers: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    /// List of data queries
    #[prost(message, repeated, tag = "3")]
    pub queries: ::prost::alloc::vec::Vec<DataQuery>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDataResponse {
    /// Map of refId to response
    #[prost(map = "string, message", tag = "1")]
    pub responses: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        DataResponse,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataResponse {
    /// Arrow encoded DataFrames
    /// Frame has its own meta, warnings, and repeats refId
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub frames: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, tag = "2")]
    pub error: ::prost::alloc::string::String,
    /// Warning: Current ignored by frontend. Would be for metadata about the query.
    #[prost(bytes = "vec", tag = "3")]
    pub json_meta: ::prost::alloc::vec::Vec<u8>,
    /// When errors exist or a non 2XX status, clients will be passed a 207 HTTP
    /// error code in /ds/query The status codes should match values from standard
    /// HTTP status codes If not set explicitly, it will be marshaled to 200 if no
    /// error exists, or 500 if one does
    #[prost(int32, tag = "4")]
    pub status: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CollectMetricsRequest {
    #[prost(message, optional, tag = "1")]
    pub plugin_context: ::core::option::Option<PluginContext>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CollectMetricsResponse {
    #[prost(message, optional, tag = "1")]
    pub metrics: ::core::option::Option<collect_metrics_response::Payload>,
}
/// Nested message and enum types in `CollectMetricsResponse`.
pub mod collect_metrics_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Payload {
        #[prost(bytes = "vec", tag = "1")]
        pub prometheus: ::prost::alloc::vec::Vec<u8>,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckHealthRequest {
    #[prost(message, optional, tag = "1")]
    pub plugin_context: ::core::option::Option<PluginContext>,
    /// Environment info
    #[prost(map = "string, string", tag = "2")]
    pub headers: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckHealthResponse {
    #[prost(enumeration = "check_health_response::HealthStatus", tag = "1")]
    pub status: i32,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub json_details: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `CheckHealthResponse`.
pub mod check_health_response {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum HealthStatus {
        Unknown = 0,
        Ok = 1,
        Error = 2,
    }
    impl HealthStatus {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                HealthStatus::Unknown => "UNKNOWN",
                HealthStatus::Ok => "OK",
                HealthStatus::Error => "ERROR",
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeStreamRequest {
    #[prost(message, optional, tag = "1")]
    pub plugin_context: ::core::option::Option<PluginContext>,
    /// path part of channel.
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    /// optional raw data. May be used as an extra payload supplied upon subscription.
    /// For example, can contain JSON query object.
    #[prost(bytes = "bytes", tag = "3")]
    pub data: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeStreamResponse {
    /// status of subscribe response.
    #[prost(enumeration = "subscribe_stream_response::Status", tag = "1")]
    pub status: i32,
    /// JSON-encoded data to return to a client in a successful
    /// subscription result.
    /// For data frame streams this can be a JSON-encoded frame schema.
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `SubscribeStreamResponse`.
pub mod subscribe_stream_response {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Status {
        Ok = 0,
        NotFound = 1,
        PermissionDenied = 2,
    }
    impl Status {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Status::Ok => "OK",
                Status::NotFound => "NOT_FOUND",
                Status::PermissionDenied => "PERMISSION_DENIED",
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublishStreamRequest {
    #[prost(message, optional, tag = "1")]
    pub plugin_context: ::core::option::Option<PluginContext>,
    /// path part of a channel.
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    /// data that user wants to publish into a stream
    /// (only JSON-encoded at the moment).
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublishStreamResponse {
    /// status of publish response.
    #[prost(enumeration = "publish_stream_response::Status", tag = "1")]
    pub status: i32,
    /// JSON-encoded data to publish into a channel. This can be
    /// unmodified data from a PublishRequest or any modified data.
    /// If empty data returned here then Grafana won't publish data
    /// to a channel itself but will return a successful result to a
    /// client (supposing plugin published data to a channel itself).
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `PublishStreamResponse`.
pub mod publish_stream_response {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Status {
        Ok = 0,
        NotFound = 1,
        PermissionDenied = 2,
    }
    impl Status {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Status::Ok => "OK",
                Status::NotFound => "NOT_FOUND",
                Status::PermissionDenied => "PERMISSION_DENIED",
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RunStreamRequest {
    #[prost(message, optional, tag = "1")]
    pub plugin_context: ::core::option::Option<PluginContext>,
    /// path part of a channel.
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    /// optional raw data. May be used as an extra payload supplied upon subscription.
    /// For example, can contain JSON query object.
    #[prost(bytes = "bytes", tag = "3")]
    pub data: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamPacket {
    /// JSON-encoded data to publish into a channel.
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// Generated client implementations.
pub mod resource_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ResourceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ResourceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ResourceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ResourceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ResourceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn call_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::CallResourceRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::CallResourceResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/pluginv2.Resource/CallResource",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod data_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct DataClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl DataClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> DataClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> DataClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            DataClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn query_data(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDataRequest>,
        ) -> Result<tonic::Response<super::QueryDataResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pluginv2.Data/QueryData");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod diagnostics_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct DiagnosticsClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl DiagnosticsClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> DiagnosticsClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> DiagnosticsClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            DiagnosticsClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn check_health(
            &mut self,
            request: impl tonic::IntoRequest<super::CheckHealthRequest>,
        ) -> Result<tonic::Response<super::CheckHealthResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/pluginv2.Diagnostics/CheckHealth",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn collect_metrics(
            &mut self,
            request: impl tonic::IntoRequest<super::CollectMetricsRequest>,
        ) -> Result<tonic::Response<super::CollectMetricsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/pluginv2.Diagnostics/CollectMetrics",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod stream_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct StreamClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl StreamClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> StreamClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> StreamClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            StreamClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// SubscribeStream called when a user tries to subscribe to a plugin/datasource
        /// managed channel path – thus plugin can check subscribe permissions and communicate
        /// options with Grafana Core. When the first subscriber joins a channel, RunStream
        /// will be called.
        pub async fn subscribe_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::SubscribeStreamRequest>,
        ) -> Result<tonic::Response<super::SubscribeStreamResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/pluginv2.Stream/SubscribeStream",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// RunStream will be initiated by Grafana to consume a stream. RunStream will be
        /// called once for the first client successfully subscribed to a channel path.
        /// When Grafana detects that there are no longer any subscribers inside a channel,
        /// the call will be terminated until next active subscriber appears. Call termination
        /// can happen with a delay.
        pub async fn run_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::RunStreamRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::StreamPacket>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/pluginv2.Stream/RunStream",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// PublishStream called when a user tries to publish to a plugin/datasource
        /// managed channel path. Here plugin can check publish permissions and
        /// modify publication data if required.
        pub async fn publish_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::PublishStreamRequest>,
        ) -> Result<tonic::Response<super::PublishStreamResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/pluginv2.Stream/PublishStream",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod resource_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ResourceServer.
    #[async_trait]
    pub trait Resource: Send + Sync + 'static {
        /// Server streaming response type for the CallResource method.
        type CallResourceStream: futures_core::Stream<
                Item = Result<super::CallResourceResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn call_resource(
            &self,
            request: tonic::Request<super::CallResourceRequest>,
        ) -> Result<tonic::Response<Self::CallResourceStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ResourceServer<T: Resource> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Resource> ResourceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ResourceServer<T>
    where
        T: Resource,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/pluginv2.Resource/CallResource" => {
                    #[allow(non_camel_case_types)]
                    struct CallResourceSvc<T: Resource>(pub Arc<T>);
                    impl<
                        T: Resource,
                    > tonic::server::ServerStreamingService<super::CallResourceRequest>
                    for CallResourceSvc<T> {
                        type Response = super::CallResourceResponse;
                        type ResponseStream = T::CallResourceStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CallResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).call_resource(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CallResourceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Resource> Clone for ResourceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Resource> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Resource> tonic::server::NamedService for ResourceServer<T> {
        const NAME: &'static str = "pluginv2.Resource";
    }
}
/// Generated server implementations.
pub mod data_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with DataServer.
    #[async_trait]
    pub trait Data: Send + Sync + 'static {
        async fn query_data(
            &self,
            request: tonic::Request<super::QueryDataRequest>,
        ) -> Result<tonic::Response<super::QueryDataResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct DataServer<T: Data> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Data> DataServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for DataServer<T>
    where
        T: Data,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/pluginv2.Data/QueryData" => {
                    #[allow(non_camel_case_types)]
                    struct QueryDataSvc<T: Data>(pub Arc<T>);
                    impl<T: Data> tonic::server::UnaryService<super::QueryDataRequest>
                    for QueryDataSvc<T> {
                        type Response = super::QueryDataResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryDataRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).query_data(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = QueryDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Data> Clone for DataServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Data> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Data> tonic::server::NamedService for DataServer<T> {
        const NAME: &'static str = "pluginv2.Data";
    }
}
/// Generated server implementations.
pub mod diagnostics_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with DiagnosticsServer.
    #[async_trait]
    pub trait Diagnostics: Send + Sync + 'static {
        async fn check_health(
            &self,
            request: tonic::Request<super::CheckHealthRequest>,
        ) -> Result<tonic::Response<super::CheckHealthResponse>, tonic::Status>;
        async fn collect_metrics(
            &self,
            request: tonic::Request<super::CollectMetricsRequest>,
        ) -> Result<tonic::Response<super::CollectMetricsResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct DiagnosticsServer<T: Diagnostics> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Diagnostics> DiagnosticsServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for DiagnosticsServer<T>
    where
        T: Diagnostics,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/pluginv2.Diagnostics/CheckHealth" => {
                    #[allow(non_camel_case_types)]
                    struct CheckHealthSvc<T: Diagnostics>(pub Arc<T>);
                    impl<
                        T: Diagnostics,
                    > tonic::server::UnaryService<super::CheckHealthRequest>
                    for CheckHealthSvc<T> {
                        type Response = super::CheckHealthResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CheckHealthRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).check_health(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CheckHealthSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/pluginv2.Diagnostics/CollectMetrics" => {
                    #[allow(non_camel_case_types)]
                    struct CollectMetricsSvc<T: Diagnostics>(pub Arc<T>);
                    impl<
                        T: Diagnostics,
                    > tonic::server::UnaryService<super::CollectMetricsRequest>
                    for CollectMetricsSvc<T> {
                        type Response = super::CollectMetricsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CollectMetricsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).collect_metrics(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CollectMetricsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Diagnostics> Clone for DiagnosticsServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Diagnostics> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Diagnostics> tonic::server::NamedService for DiagnosticsServer<T> {
        const NAME: &'static str = "pluginv2.Diagnostics";
    }
}
/// Generated server implementations.
pub mod stream_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with StreamServer.
    #[async_trait]
    pub trait Stream: Send + Sync + 'static {
        /// SubscribeStream called when a user tries to subscribe to a plugin/datasource
        /// managed channel path – thus plugin can check subscribe permissions and communicate
        /// options with Grafana Core. When the first subscriber joins a channel, RunStream
        /// will be called.
        async fn subscribe_stream(
            &self,
            request: tonic::Request<super::SubscribeStreamRequest>,
        ) -> Result<tonic::Response<super::SubscribeStreamResponse>, tonic::Status>;
        /// Server streaming response type for the RunStream method.
        type RunStreamStream: futures_core::Stream<
                Item = Result<super::StreamPacket, tonic::Status>,
            >
            + Send
            + 'static;
        /// RunStream will be initiated by Grafana to consume a stream. RunStream will be
        /// called once for the first client successfully subscribed to a channel path.
        /// When Grafana detects that there are no longer any subscribers inside a channel,
        /// the call will be terminated until next active subscriber appears. Call termination
        /// can happen with a delay.
        async fn run_stream(
            &self,
            request: tonic::Request<super::RunStreamRequest>,
        ) -> Result<tonic::Response<Self::RunStreamStream>, tonic::Status>;
        /// PublishStream called when a user tries to publish to a plugin/datasource
        /// managed channel path. Here plugin can check publish permissions and
        /// modify publication data if required.
        async fn publish_stream(
            &self,
            request: tonic::Request<super::PublishStreamRequest>,
        ) -> Result<tonic::Response<super::PublishStreamResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct StreamServer<T: Stream> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Stream> StreamServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for StreamServer<T>
    where
        T: Stream,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/pluginv2.Stream/SubscribeStream" => {
                    #[allow(non_camel_case_types)]
                    struct SubscribeStreamSvc<T: Stream>(pub Arc<T>);
                    impl<
                        T: Stream,
                    > tonic::server::UnaryService<super::SubscribeStreamRequest>
                    for SubscribeStreamSvc<T> {
                        type Response = super::SubscribeStreamResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SubscribeStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).subscribe_stream(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubscribeStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/pluginv2.Stream/RunStream" => {
                    #[allow(non_camel_case_types)]
                    struct RunStreamSvc<T: Stream>(pub Arc<T>);
                    impl<
                        T: Stream,
                    > tonic::server::ServerStreamingService<super::RunStreamRequest>
                    for RunStreamSvc<T> {
                        type Response = super::StreamPacket;
                        type ResponseStream = T::RunStreamStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RunStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).run_stream(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RunStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/pluginv2.Stream/PublishStream" => {
                    #[allow(non_camel_case_types)]
                    struct PublishStreamSvc<T: Stream>(pub Arc<T>);
                    impl<
                        T: Stream,
                    > tonic::server::UnaryService<super::PublishStreamRequest>
                    for PublishStreamSvc<T> {
                        type Response = super::PublishStreamResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PublishStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).publish_stream(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PublishStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Stream> Clone for StreamServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Stream> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Stream> tonic::server::NamedService for StreamServer<T> {
        const NAME: &'static str = "pluginv2.Stream";
    }
}
