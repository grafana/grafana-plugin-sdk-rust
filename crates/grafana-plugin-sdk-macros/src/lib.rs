use proc_macro::TokenStream;
use proc_macro2::Span;

use darling::{FromDeriveInput, FromMeta};
use quote::{quote, quote_spanned, ToTokens};
use syn::{meta::ParseNestedMeta, parenthesized, parse::ParseBuffer, parse_macro_input, Lit};

#[derive(Debug, FromMeta)]
enum PluginType {
    Datasource,
    App,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(grafana_plugin))]
struct GrafanaPluginOpts {
    ident: syn::Ident,
    plugin_type: PluginType,
    json_data: Option<syn::Path>,
    secure_json_data: Option<syn::Path>,
}

#[doc(hidden)]
#[allow(missing_docs)]
#[proc_macro_derive(GrafanaPlugin, attributes(grafana_plugin))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("Couldn't parse item");
    let GrafanaPluginOpts {
        ident,
        plugin_type,
        json_data,
        secure_json_data,
    } = match GrafanaPluginOpts::from_derive_input(&ast) {
        Ok(x) => x,
        Err(e) => return e.flatten().write_errors().into(),
    };
    let ptype = match plugin_type {
        PluginType::App => {
            quote! { ::grafana_plugin_sdk::backend::AppPlugin<Self::JsonData, Self::SecureJsonData> }
        }
        PluginType::Datasource => {
            quote! { ::grafana_plugin_sdk::backend::DataSourcePlugin<Self::JsonData, Self::SecureJsonData> }
        }
    };
    let json_data =
        json_data.unwrap_or_else(|| syn::parse_quote!(::grafana_plugin_sdk::serde_json::Value));
    let secure_json_data = secure_json_data
        .unwrap_or_else(|| syn::parse_quote!(::grafana_plugin_sdk::serde_json::Value));
    quote! {
        impl ::grafana_plugin_sdk::backend::GrafanaPlugin for #ident {

            type PluginType = #ptype;
            type JsonData = #json_data;
            type SecureJsonData = #secure_json_data;
        }
    }
    .into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}

#[derive(Default)]
struct Configuration {
    services: Option<Services>,
    init_subscriber: Option<bool>,
    shutdown_handler: Option<String>,
}

impl Configuration {
    fn set_services(&mut self, meta: ParseNestedMeta) -> Result<(), syn::Error> {
        if self.services.is_some() {
            return Err(meta.error("`services` set multiple times."));
        }
        let mut cfg_services = Services::default();
        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("data") {
                if cfg_services.data {
                    return Err(meta.error("`data` set multiple times."));
                }
                cfg_services.data = true;
                Ok(())
            } else if meta.path.is_ident("diagnostics") {
                if cfg_services.diagnostics {
                    return Err(meta.error("`diagnostics` set multiple times."));
                }
                cfg_services.diagnostics = true;
                Ok(())
            } else if meta.path.is_ident("resource") {
                if cfg_services.resource {
                    return Err(meta.error("`resource` set multiple times."));
                }
                cfg_services.resource = true;
                Ok(())
            } else if meta.path.is_ident("stream") {
                if cfg_services.stream {
                    return Err(meta.error("`stream` set multiple times."));
                }
                cfg_services.stream = true;
                Ok(())
            } else {
                Err(meta.error(
                    "Unknown service. Only `data`, `diagnostics`, `resource` and `stream` are supported.",
                ))
            }
        })?;
        if !cfg_services.data
            && !cfg_services.diagnostics
            && !cfg_services.resource
            && !cfg_services.stream
        {
            return Err(meta.error("At least one service must be specified in `services`."));
        }

        self.services = Some(cfg_services);
        Ok(())
    }

    fn get_accidental_nested_meta(input: &ParseBuffer) -> Result<Lit, syn::Error> {
        let content;
        parenthesized!(content in input);
        let x: Lit = content.parse()?;
        Ok(x)
    }

    fn set_init_subscriber(&mut self, init_subscriber: ParseNestedMeta) -> Result<(), syn::Error> {
        if self.init_subscriber.is_some() {
            return Err(init_subscriber.error("`init_subscriber` set multiple times."));
        }
        let value = init_subscriber.value().map_err(|_| {
            init_subscriber.error(format!(
                "`init_subscriber` should be specified as `init_subscriber = {}`",
                Self::get_accidental_nested_meta(init_subscriber.input)
                    .ok()
                    .and_then(|x| if let Lit::Bool(b) = x {
                        Some(b.value)
                    } else {
                        None
                    })
                    .unwrap_or(true)
            ))
        })?;
        let s: syn::LitBool = value
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "`init_subscriber` must be a bool literal."))?;
        self.init_subscriber = Some(s.value);
        Ok(())
    }

    fn set_shutdown_handler(
        &mut self,
        shutdown_handler: ParseNestedMeta,
    ) -> Result<(), syn::Error> {
        if self.shutdown_handler.is_some() {
            return Err(shutdown_handler.error("`shutdown_handler` set multiple times."));
        }
        let value = shutdown_handler.value().map_err(|_| {
            let address = Self::get_accidental_nested_meta(shutdown_handler.input)
                .ok()
                .and_then(|x| {
                    if let Lit::Str(s) = x {
                        Some(s.value())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "<address>".to_string());
            shutdown_handler.error(format!(
                r#"`shutdown_handler` should be specified as `shutdown_handler = "{}""#,
                address
            ))
        })?;
        let s: syn::LitStr = value.parse().map_err(|e| {
            syn::Error::new(e.span(), "`shutdown_handler` must be a string literal.")
        })?;
        self.shutdown_handler = Some(s.value());
        Ok(())
    }

    fn parse(&mut self, meta: ParseNestedMeta) -> Result<(), syn::Error> {
        if meta.path.is_ident("init_subscriber") {
            self.set_init_subscriber(meta)?;
        } else if meta.path.is_ident("shutdown_handler") {
            self.set_shutdown_handler(meta)?;
        } else if meta.path.is_ident("services") {
            self.set_services(meta)?;
        } else {
            return Err(meta.error("Unknown attribute. Only `services`, `init_subscriber` and `shutdown_handler` are supported."));
        }
        Ok(())
    }

    fn build(self) -> FinalConfig {
        FinalConfig {
            services: self.services.unwrap(),
            init_subscriber: self.init_subscriber.unwrap_or_default(),
            shutdown_handler: self.shutdown_handler,
        }
    }
}

#[derive(Default)]
struct Services {
    stream: bool,
    data: bool,
    diagnostics: bool,
    resource: bool,
}

#[derive(Default)]
struct FinalConfig {
    services: Services,
    init_subscriber: bool,
    shutdown_handler: Option<String>,
}

/// Config used in case of the attribute not being able to build a valid config
const DEFAULT_ERROR_CONFIG: FinalConfig = FinalConfig {
    services: Services {
        stream: false,
        data: false,
        diagnostics: false,
        resource: false,
    },
    init_subscriber: false,
    shutdown_handler: None,
};

fn parse_knobs(input: syn::ItemFn, config: FinalConfig) -> TokenStream {
    // If type mismatch occurs, the current rustc points to the last statement.
    let (last_stmt_start_span, _) = {
        let mut last_stmt = input
            .block
            .stmts
            .last()
            .map(ToTokens::into_token_stream)
            .unwrap_or_default()
            .into_iter();
        // `Span` on stable Rust has a limitation that only points to the first
        // token, not the whole tokens. We can work around this limitation by
        // using the first/last span of the tokens like
        // `syn::Error::new_spanned` does.
        let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
        let end = last_stmt.last().map_or(start, |t| t.span());
        (start, end)
    };

    let body = input.block;

    let mut plugin = quote_spanned! {last_stmt_start_span=>
        ::grafana_plugin_sdk::backend::Plugin::new()
    };
    if config.services.data {
        plugin = quote! { #plugin.data_service(std::clone::Clone::clone(&service)) };
    }
    if config.services.diagnostics {
        plugin = quote! { #plugin.diagnostics_service(std::clone::Clone::clone(&service)) };
    }
    if config.services.resource {
        plugin = quote! { #plugin.resource_service(std::clone::Clone::clone(&service)) };
    }
    if config.services.stream {
        plugin = quote! { #plugin.stream_service(std::clone::Clone::clone(&service)) };
    }
    let init_subscriber = config.init_subscriber;
    if init_subscriber {
        plugin = quote! { #plugin.init_subscriber(#init_subscriber) };
    }
    if let Some(x) = config.shutdown_handler {
        let shutdown_handler =
            quote! { #x.parse().expect("could not parse shutdown handler as SocketAddr") };
        plugin = quote! { #plugin.shutdown_handler(#shutdown_handler) };
    }

    let expanded = quote! {
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            let fut = async {
                let listener = ::grafana_plugin_sdk::backend::initialize().await?;
                let service = #body;
                #plugin
                    .start(listener)
                    .await?;
                Ok::<_, Box<dyn std::error::Error>>(())
            };
            ::grafana_plugin_sdk::async_main(fut)?;
            Ok(())
        }
    };
    TokenStream::from(expanded)
}

/**
Generates a `main` function that starts a [`Plugin`](../grafana_plugin_sdk/backend/struct.Plugin.html) with the returned service struct.

When applied to a function that returns a struct implementing one or more of the various
`Service` traits, `#[main]` will create an async runtime and a [`Plugin`](../grafana_plugin_sdk/backend/struct.Plugin.html),
then attach the desired services

The returned struct _must_ be `Clone` so that it can be used to handle multiple
services.

# Attributes

## `services`

The `services` attribute takes a list of services that the plugin should expose.
At least one service must be specified. Possible options are:

- `data` (registers a [`DataService`](../grafana_plugin_sdk/backend/trait.DataService.html) using [`Plugin::data_service`](../grafana_plugin_sdk/backend/struct.Plugin.html#method.data_service))
- `diagnostics` (registers a [`DiagnosticsService`](../grafana_plugin_sdk/backend/trait.DiagnosticsService.html) using [`Plugin::data_service`](../grafana_plugin_sdk/backend/struct.Plugin.html#method.diagnostics_service))
- `resource` (registers a [`ResourceService`](../grafana_plugin_sdk/backend/trait.ResourceService.html) using [`Plugin::data_service`](../grafana_plugin_sdk/backend/struct.Plugin.html#method.resource_service))
- `stream` (registers a [`StreamService`](../grafana_plugin_sdk/backend/trait.StreamService.html) using [`Plugin::data_service`](../grafana_plugin_sdk/backend/struct.Plugin.html#method.stream_service))

### Example:

```rust
# use std::sync::Arc;
#
# use grafana_plugin_sdk::{backend, data, prelude::*};
# use serde::Deserialize;
# use thiserror::Error;
#
# #[derive(Clone, GrafanaPlugin)]
# #[grafana_plugin(plugin_type = "app")]
# struct Plugin;
#
# #[derive(Debug, Deserialize)]
# struct Query {
#     pub expression: String,
#     pub other_user_input: u64,
# }
#
# #[derive(Debug)]
# struct QueryError {
#     source: data::Error,
#     ref_id: String,
# }
#
# impl std::fmt::Display for QueryError {
#     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
#         write!(f, "Error in query {}: {}", self.ref_id, self.source)
#     }
# }
#
# impl std::error::Error for QueryError {}
#
# impl backend::DataQueryError for QueryError {
#     fn ref_id(self) -> String {
#         self.ref_id
#     }
# }
#
# #[backend::async_trait]
# impl backend::DataService for Plugin {
#     type Query = Query;
#     type QueryError = QueryError;
#     type Stream = backend::BoxDataResponseStream<Self::QueryError>;
#     async fn query_data(&self, request: backend::QueryDataRequest<Self::Query, Self>) -> Self::Stream {
#         todo!()
#     }
# }
#
# #[backend::async_trait]
# impl backend::DiagnosticsService for Plugin {
#     type CheckHealthError = std::convert::Infallible;
#
#     async fn check_health(
#         &self,
#         request: backend::CheckHealthRequest<Self>,
#     ) -> Result<backend::CheckHealthResponse, Self::CheckHealthError> {
#         todo!()
#     }
#
#     type CollectMetricsError = Arc<dyn std::error::Error + Send + Sync>;
#
#     async fn collect_metrics(
#         &self,
#         request: backend::CollectMetricsRequest<Self>,
#     ) -> Result<backend::CollectMetricsResponse, Self::CollectMetricsError> {
#         todo!()
#     }
# }
#
# #[derive(Debug, Error)]
# enum ResourceError {
#     #[error("HTTP error: {0}")]
#     Http(#[from] http::Error),
#
#     #[error("Path not found")]
#     NotFound,
# }
#
# impl backend::ErrIntoHttpResponse for ResourceError {}
#
# #[backend::async_trait]
# impl backend::ResourceService for Plugin {
#     type Error = ResourceError;
#     type InitialResponse = Vec<u8>;
#     type Stream = backend::BoxResourceStream<Self::Error>;
#     async fn call_resource(&self, r: backend::CallResourceRequest<Self>) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
#         todo!()
#     }
# }
#
# #[backend::async_trait]
# impl backend::StreamService for Plugin {
#     type JsonValue = ();
#     async fn subscribe_stream(
#         &self,
#         request: backend::SubscribeStreamRequest<Self>,
#     ) -> Result<backend::SubscribeStreamResponse, Self::Error> {
#         todo!()
#     }
#     type Error = Arc<dyn std::error::Error>;
#     type Stream = backend::BoxRunStream<Self::Error>;
#     async fn run_stream(&self, _request: backend::RunStreamRequest<Self>) -> Result<Self::Stream, Self::Error> {
#         todo!()
#     }
#     async fn publish_stream(
#         &self,
#         _request: backend::PublishStreamRequest<Self>,
#     ) -> Result<backend::PublishStreamResponse, Self::Error> {
#         todo!()
#     }
# }
#[grafana_plugin_sdk::main(
    services(data, diagnostics, resource, stream),
)]
async fn plugin() -> Plugin {
    Plugin
}
```

## `init_subscriber`

The `init_subscriber` attribute indicates whether a tracing subscriber should be
initialized automatically using
[`Plugin::init_subscriber`](../grafana_plugin_sdk/backend/struct.Plugin.html#method.init_subscriber).
Unless this is being done in the annotated plugin function, this should
generally be set to `true`.

This must be a boolean.

### Example

```
# use std::sync::Arc;
#
# use grafana_plugin_sdk::{backend, prelude::*};
# use thiserror::Error;
#
# #[derive(Clone, GrafanaPlugin)]
# #[grafana_plugin(plugin_type = "app")]
# struct Plugin;
#
# #[derive(Debug, Error)]
# enum ResourceError {
#     #[error("HTTP error: {0}")]
#     Http(#[from] http::Error),
#
#     #[error("Path not found")]
#     NotFound,
# }
#
# impl backend::ErrIntoHttpResponse for ResourceError {}
#
# #[backend::async_trait]
# impl backend::ResourceService for Plugin {
#     type Error = ResourceError;
#     type InitialResponse = Vec<u8>;
#     type Stream = backend::BoxResourceStream<Self::Error>;
#     async fn call_resource(&self, r: backend::CallResourceRequest<Self>) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
#         todo!()
#     }
# }
#
#[grafana_plugin_sdk::main(
    services(resource),
    init_subscriber = true,
)]
async fn plugin() -> Plugin {
    Plugin
}
```

## `shutdown_handler`

The `shutdown_handler` attribute indicates that a shutdown handler should be exposed using
[`Plugin::shutdown_handler`](../grafana_plugin_sdk/backend/struct.Plugin.html#method.shutdown_handler)

This must be a string which can be parsed as a [`SocketAddr`][std::net::SocketAddr] using `SocketAddr::parse`.

### Example

```
# use std::sync::Arc;
#
# use grafana_plugin_sdk::{backend, prelude::*};
# use thiserror::Error;
#
# #[derive(Clone, GrafanaPlugin)]
# #[grafana_plugin(plugin_type = "app")]
# struct Plugin;
#
# #[derive(Debug, Error)]
# enum ResourceError {
#     #[error("HTTP error: {0}")]
#     Http(#[from] http::Error),
#
#     #[error("Path not found")]
#     NotFound,
# }
#
# impl backend::ErrIntoHttpResponse for ResourceError {}
#
# #[backend::async_trait]
# impl backend::ResourceService for Plugin {
#     type Error = ResourceError;
#     type InitialResponse = Vec<u8>;
#     type Stream = backend::BoxResourceStream<Self::Error>;
#     async fn call_resource(&self, r: backend::CallResourceRequest<Self>) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
#         todo!()
#     }
# }
#
#[grafana_plugin_sdk::main(
    services(resource),
    shutdown_handler = "127.0.0.1:10001",
)]
async fn plugin() -> Plugin {
    Plugin
}
```

# Macro expansion

The following example shows what the `#[main]` macro expands to:

```rust
use std::sync::Arc;

use grafana_plugin_sdk::{backend, prelude::*};
use thiserror::Error;

# #[derive(Clone, GrafanaPlugin)]
# #[grafana_plugin(plugin_type = "app")]
struct Plugin;

#[derive(Debug, Error)]
enum ResourceError {
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),

    #[error("Path not found")]
    NotFound,
}

impl backend::ErrIntoHttpResponse for ResourceError {}

#[backend::async_trait]
impl backend::ResourceService for Plugin {
    type Error = ResourceError;
    type InitialResponse = Vec<u8>;
    type Stream = backend::BoxResourceStream<Self::Error>;
    async fn call_resource(&self, r: backend::CallResourceRequest<Self>) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
        todo!()
    }
}

#[grafana_plugin_sdk::main(
    services(resource),
    init_subscriber = true,
    shutdown_handler = "127.0.0.1:10001",
)]
async fn plugin() -> Plugin {
    Plugin
}
```

expands to:

```rust
# use std::sync::Arc;
#
# use grafana_plugin_sdk::{backend, prelude::*};
# use thiserror::Error;
#
# #[derive(Clone, GrafanaPlugin)]
# #[grafana_plugin(plugin_type = "app")]
# struct Plugin;
#
# #[derive(Debug, Error)]
# enum ResourceError {
#     #[error("HTTP error: {0}")]
#     Http(#[from] http::Error),
#
#     #[error("Path not found")]
#     NotFound,
# }
#
# impl backend::ErrIntoHttpResponse for ResourceError {}
#
# #[backend::async_trait]
# impl backend::ResourceService for Plugin {
#     type Error = ResourceError;
#     type InitialResponse = Vec<u8>;
#     type Stream = backend::BoxResourceStream<Self::Error>;
#     async fn call_resource(&self, r: backend::CallResourceRequest<Self>) -> Result<(Self::InitialResponse, Self::Stream), Self::Error> {
#         todo!()
#     }
# }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fut = async {
        let listener = ::grafana_plugin_sdk::backend::initialize().await?;
        let service = Plugin;
        ::grafana_plugin_sdk::backend::Plugin::new()
            .resource_service(service.clone())
            .init_subscriber(true)
            .shutdown_handler("127.0.0.1:10001".parse().expect("could not parse shutdown handler as SocketAddr"))
            .start(listener)
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    };
    # if false {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("grafana-plugin-worker-thread")
        .enable_all()
        .build()
        .expect("create tokio runtime")
        .block_on(fut)?;
    # }
    Ok(())
}
```
*/
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };
    if args.is_empty() {
        let span = Span::call_site();
        let source_text = span
            .source_text()
            .unwrap_or_else(|| "grafana_plugin_sdk::main".to_string());
        return token_stream_with_error(
            parse_knobs(input, DEFAULT_ERROR_CONFIG),
            syn::Error::new(
                span,
                format!(
                    "at least one service must be provided, e.g. `#[{}(services(data))]`",
                    source_text.trim_matches(&['[', ']', '#'] as &[char])
                ),
            ),
        );
    }

    let res = if input.sig.ident != "plugin" {
        let msg = "the plugin function must be named 'plugin'";
        Err(syn::Error::new_spanned(&input.sig.ident, msg))
    } else if !input.sig.inputs.is_empty() {
        let msg = "the plugin function cannot accept arguments";
        Err(syn::Error::new_spanned(&input.sig.inputs, msg))
    } else if input.sig.asyncness.is_none() {
        let msg = "the `async` keyword is missing from the function declaration";
        Err(syn::Error::new_spanned(input.sig.fn_token, msg))
    } else {
        let mut config = Configuration::default();
        let config_parser = syn::meta::parser(|meta| config.parse(meta));
        parse_macro_input!(args with config_parser);
        Ok(config)
    };

    match res {
        Ok(c) => parse_knobs(input, c.build()),
        Err(e) => token_stream_with_error(parse_knobs(input, DEFAULT_ERROR_CONFIG), e),
    }
}
