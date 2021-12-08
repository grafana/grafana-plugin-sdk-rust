//! Resource services, which allow backend plugins to handle custom HTTP requests and responses.
use std::pin::Pin;

use futures_util::StreamExt;
use http::{
    header::{HeaderName, HeaderValue},
    Request, Response, StatusCode,
};
use itertools::Itertools;
use prost::bytes::Bytes;

use crate::{
    backend::{ConvertFromError, ConvertToError, PluginContext},
    pluginv2,
};

/// A request for a resource call.
#[derive(Debug)]
pub struct CallResourceRequest<T = Bytes> {
    /// Details of the plugin instance from which the request originated.
    pub plugin_context: Option<PluginContext>,
    /// The HTTP request.
    pub request: Request<T>,
}

impl TryFrom<pluginv2::CallResourceRequest> for CallResourceRequest {
    type Error = ConvertFromError;
    fn try_from(other: pluginv2::CallResourceRequest) -> Result<Self, Self::Error> {
        let mut request = Request::builder();
        {
            let headers = request
                .headers_mut()
                .expect("Request should always be valid at this point");
            for (k, vs) in other.headers.iter() {
                let header_name =
                    k.parse::<HeaderName>()
                        .map_err(|source| ConvertFromError::InvalidRequest {
                            source: source.into(),
                        })?;
                for v in &vs.values {
                    let header_val = v.parse::<HeaderValue>().map_err(|source| {
                        ConvertFromError::InvalidRequest {
                            source: source.into(),
                        }
                    })?;
                    headers.append(header_name.clone(), header_val);
                }
            }
        }
        Ok(Self {
            plugin_context: other.plugin_context.map(TryInto::try_into).transpose()?,
            request: request
                .method(other.method.as_str())
                .uri(other.url)
                .body(other.body)
                .map_err(|source| ConvertFromError::InvalidRequest { source })?,
        })
    }
}

impl TryFrom<Response<Bytes>> for pluginv2::CallResourceResponse {
    type Error = ConvertToError;
    fn try_from(mut other: Response<Bytes>) -> Result<Self, Self::Error> {
        let grouped_headers = other.headers_mut().drain().group_by(|x| x.0.clone());
        let headers = grouped_headers
            .into_iter()
            .map(|(k, values)| {
                Ok((
                    k.as_ref()
                        .map(|h| h.to_string())
                        .ok_or(ConvertToError::InvalidResponse)?,
                    pluginv2::StringList {
                        values: values
                            .map(|v| {
                                v.1.to_str()
                                    .map(|s| s.to_string())
                                    .map_err(|_| ConvertToError::InvalidResponse)
                            })
                            .collect::<Result<_, _>>()?,
                    },
                ))
            })
            .collect::<Result<_, _>>()?;
        drop(grouped_headers);
        Ok(Self {
            code: other.status().as_u16().into(),
            headers,
            body: other.into_body(),
        })
    }
}

fn body_to_response(body: Bytes) -> pluginv2::CallResourceResponse {
    pluginv2::CallResourceResponse {
        code: 200,
        headers: std::collections::HashMap::new(),
        body,
    }
}

/// Type alias for a pinned, boxed future with a fallible HTTP response as output, with a custom error type.
pub type BoxResourceFuture<E> =
    Pin<Box<dyn std::future::Future<Output = Result<Response<Bytes>, E>>>>;

/// Type alias for a pinned, boxed stream of HTTP responses with a custom error type.
pub type BoxResourceStream<E> = Pin<Box<dyn futures_core::Stream<Item = Result<Bytes, E>> + Send>>;

/// Trait for plugins that can handle arbitrary resource requests.
///
/// Implementing this trait allows plugins to handle a wide variety of use cases beyond
/// 'just' responding to requests for data and returning dataframes.
///
/// See <https://grafana.com/docs/grafana/latest/developers/plugins/backend/#resources> for
/// some examples of how this can be used.
///
/// # Examples
///
/// ## Simple function
///
/// This trait has a blanket impl for async functions taking a `CallResourceRequest` and
/// returning a `Result<T, E> where T: IntoHttpResponse + Send, E: std::error::Error + Send`.
/// [`IntoHttpResponse`] is implemented for some types already - see its docs for details.
/// Note that the `reqwest` feature of this crate is required for the `IntoHttpResponse`
/// implementation to be enabled for [`reqwest::Response`].
#[cfg_attr(
    feature = "reqwest",
    doc = r##"
Note that this example requires the `reqwest` feature.

```rust
# extern crate reqwest_lib as reqwest;
use bytes::Bytes;
use grafana_plugin_sdk::{backend, prelude::*};
use reqwest::{Error, Response};

async fn respond(req: backend::CallResourceRequest) -> Result<Response, Error> {
    reqwest::get("https://www.rust-lang.org").await
}

let plugin = backend::Plugin::new().resource_service(respond);
```
"##
)]
/// ## Stateful service
///
/// The following shows an example implementation of [`ResourceService`] which handles
/// two endpoints:
/// - /echo, which echos back the request's URL, headers and body in three responses,
/// - /count, which increments the plugin's internal count and returns it in a response.
///
/// ```rust
/// use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
///
/// use async_stream::stream;
/// use bytes::Bytes;
/// use grafana_plugin_sdk::backend;
/// use http::Response;
/// use thiserror::Error;
///
/// struct Plugin(Arc<AtomicUsize>);
///
/// impl Plugin {
///     // Increment the counter and return the stringified result in a `Response`.
///     fn inc_and_respond(&self) -> Response<Bytes> {
///         Response::new(
///             self.0
///                 .fetch_add(1, Ordering::SeqCst)
///                 .to_string()
///                 .into_bytes()
///                 .into()
///         )
///     }
/// }
///
/// #[derive(Debug, Error)]
/// enum ResourceError {
///     #[error("HTTP error: {0}")]
///     Http(#[from] http::Error),
///
///     #[error("Path not found")]
///     NotFound,
/// }
///
/// impl backend::ErrIntoHttpResponse for ResourceError {}
///
/// #[backend::async_trait]
/// impl backend::ResourceService for Plugin {
///     type Error = ResourceError;
///     type InitialResponse = Response<Bytes>;
///     type Stream = backend::BoxResourceStream<Self::Error>;
///     async fn call_resource(&self, r: backend::CallResourceRequest) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
///         let count = Arc::clone(&self.0);
///         let response_and_stream = match r.request.uri().path() {
///             // Just send back a single response.
///             "/echo" => Ok((
///                 Response::new(r.request.into_body().into()),
///                 Box::pin(futures::stream::empty()) as Self::Stream,
///             )),
///             // Send an initial response with the current count, then stream the gradually
///             // incrementing count back to the client.
///             "/count" => Ok((
///                 Response::new(
///                     count
///                         .fetch_add(1, Ordering::SeqCst)
///                         .to_string()
///                         .into_bytes()
///                         .into(),
///                 ),
///                 Box::pin(async_stream::try_stream! {
///                     loop {
///                         let body = count
///                             .fetch_add(1, Ordering::SeqCst)
///                             .to_string()
///                             .into_bytes()
///                             .into();
///                         yield body;
///                     }
///                 }) as Self::Stream,
///             )),
///             _ => return Err(ResourceError::NotFound),
///         };
///         response_and_stream
///     }
/// }
/// ```
#[tonic::async_trait]
pub trait ResourceService {
    /// The error type that can be returned by individual responses.
    type Error: std::error::Error + ErrIntoHttpResponse + Send;

    /// The type returned as the initial response returned back to Grafana.
    ///
    /// This must be convertable into a `http::Response<Bytes>`.
    type InitialResponse: IntoHttpResponse + Send;

    /// The type of stream of optional additional data returned by `run_stream`.
    ///
    /// This will generally be impossible to name directly, so returning the
    /// [`BoxResourceStream`] type alias will probably be more convenient.
    type Stream: futures_core::Stream<Item = Result<Bytes, Self::Error>> + Send;

    /// Handle a resource request.
    ///
    /// It is completely up to the implementor how to handle the incoming request.
    ///
    /// A stream of responses can be returned. A simple way to return just a single response
    /// is to use [`futures_util::stream::once`].
    async fn call_resource(
        &self,
        request: CallResourceRequest,
    ) -> Result<(Self::InitialResponse, Self::Stream), Self::Error>;
}

#[tonic::async_trait]
impl<T> pluginv2::resource_server::Resource for T
where
    T: ResourceService + Send + Sync + 'static,
{
    type CallResourceStream = Pin<
        Box<
            dyn futures_core::Stream<Item = Result<pluginv2::CallResourceResponse, tonic::Status>>
                + Send,
        >,
    >;

    #[tracing::instrument(skip(self), level = "debug")]
    async fn call_resource(
        &self,
        request: tonic::Request<pluginv2::CallResourceRequest>,
    ) -> Result<tonic::Response<Self::CallResourceStream>, tonic::Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(ConvertFromError::into_tonic_status)?;
        let (initial_response, stream) = match ResourceService::call_resource(self, request)
            .await
            .map_err(|e: <Self as ResourceService>::Error| e.into_http_response())
        {
            Ok(x) => x,
            Err(resp) => {
                let response = resp.map_err(|e| tonic::Status::internal(e.to_string()))?;
                let response = pluginv2::CallResourceResponse::try_from(response)
                    .map_err(|e| tonic::Status::internal(e.to_string()));
                let stream = futures_util::stream::once(futures_util::future::ready(response));
                return Ok(tonic::Response::new(Box::pin(stream)));
            }
        };
        let initial_response_converted = initial_response
            .into_http_response()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))
            .and_then(|response| {
                pluginv2::CallResourceResponse::try_from(response)
                    .map_err(|e| tonic::Status::internal(e.to_string()))
            });
        let stream =
            futures_util::stream::once(futures_util::future::ready(initial_response_converted))
                .chain(stream.map(|response| {
                    response
                        .map(body_to_response)
                        .map_err(|e| tonic::Status::internal(e.to_string()))
                }));
        Ok(tonic::Response::new(Box::pin(stream)))
    }
}

#[tonic::async_trait]
impl<T, F, R, E> ResourceService for T
where
    T: Fn(CallResourceRequest) -> F + Sync,
    F: std::future::Future<Output = Result<R, E>> + Send,
    R: IntoHttpResponse + Send + Sync,
    E: std::error::Error + ErrIntoHttpResponse + Send + Sync,
{
    type Error = E;
    type InitialResponse = R;
    type Stream = futures_util::stream::Empty<Result<Bytes, Self::Error>>;
    async fn call_resource(
        &self,
        request: CallResourceRequest,
    ) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
        let response = self(request).await?;
        Ok((response, futures_util::stream::empty()))
    }
}

/// Trait indicating that a type can be fallibly converted into a `http::Response<Bytes>`.
///
/// This is a separate trait (rather than using `TryFrom`/`TryInto`) because it may
/// need to be async.
#[tonic::async_trait]
pub trait IntoHttpResponse {
    /// Performs the conversion.
    async fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>>;
}

#[tonic::async_trait]
impl IntoHttpResponse for http::Response<Bytes> {
    async fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
        Ok(self)
    }
}

#[cfg(feature = "reqwest")]
#[tonic::async_trait]
impl IntoHttpResponse for reqwest::Response {
    async fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
        Ok(self.bytes().await.map(http::Response::new)?)
    }
}

#[tonic::async_trait]
impl IntoHttpResponse for Vec<u8> {
    async fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
        Ok(http::Response::new(self.into()))
    }
}

#[tonic::async_trait]
impl IntoHttpResponse for serde_json::Value {
    async fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
        Ok(serde_json::to_vec(&self).map(|v| http::Response::new(v.into()))?)
    }
}

/// Trait describing how an error should be converted into a `http::Response<Bytes>`.
pub trait ErrIntoHttpResponse: std::error::Error + Sized {
    /// Convert this error into a HTTP response.
    ///
    /// The default implementation returns a response with status code 500 (Internal Server Error)
    /// and the `Display` implementation of `Self` inside the `"error"` field of a JSON object
    /// in the body.
    ///
    /// Implementors may wish to override this if they wish to provide an alternative status code
    /// depending on, for example, the type of error returned from a resource call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bytes::Bytes;
    /// use grafana_plugin_sdk::backend;
    /// use thiserror::Error;
    ///
    /// #[derive(Debug, Error)]
    /// enum ResourceError {
    ///     #[error("HTTP error: {0}")]
    ///     Http(#[from] http::Error),
    ///
    ///     #[error("Path not found")]
    ///     NotFound,
    /// }
    ///
    /// impl backend::ErrIntoHttpResponse for ResourceError {
    ///     fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
    ///         let status = match &self {
    ///             Self::Http(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    ///             Self::NotFound => http::StatusCode::NOT_FOUND,
    ///         };
    ///         Ok(http::Response::builder()
    ///             .status(status)
    ///             .body(Bytes::from(serde_json::to_vec(
    ///                 &serde_json::json!({"error": self.to_string()}),
    ///             )?))?)
    ///     }
    /// }
    /// ```
    fn into_http_response(self) -> Result<http::Response<Bytes>, Box<dyn std::error::Error>> {
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Bytes::from(serde_json::to_vec(
                &serde_json::json!({"error": self.to_string()}),
            )?))?)
    }
}

impl ErrIntoHttpResponse for std::convert::Infallible {}
