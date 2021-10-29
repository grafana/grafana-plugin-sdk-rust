use proc_macro::TokenStream;
use proc_macro2::Span;

use quote::{quote, quote_spanned, ToTokens};
use syn::parse::Parser;

// syn::AttributeArgs does not implement syn::Parse
type AttributeArgs = syn::punctuated::Punctuated<syn::NestedMeta, syn::Token![,]>;

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
    fn new() -> Self {
        Self::default()
    }

    fn set_services(&mut self, services: &AttributeArgs, span: Span) -> Result<(), syn::Error> {
        if self.services.is_some() {
            return Err(syn::Error::new(span, "`services` set multiple times."));
        }
        let services = parse_services(&services)?;
        let mut cfg_services = Services::default();
        for service in services {
            if service.as_str() == "data" {
                cfg_services.data = true;
            }
            if service.as_str() == "diagnostics" {
                cfg_services.diagnostics = true;
            }
            if service.as_str() == "resource" {
                cfg_services.resource = true;
            }
            if service.as_str() == "stream" {
                cfg_services.stream = true;
            }
        }
        self.services = Some(cfg_services);
        Ok(())
    }

    fn set_init_subscriber(
        &mut self,
        init_subscriber: syn::Lit,
        span: Span,
    ) -> Result<(), syn::Error> {
        if self.init_subscriber.is_some() {
            return Err(syn::Error::new(
                span,
                "`init_subscriber` set multiple times.",
            ));
        }
        let init_subscriber = parse_bool(init_subscriber, span, "init_subscriber")?;
        self.init_subscriber = Some(init_subscriber);
        Ok(())
    }

    fn set_shutdown_handler(
        &mut self,
        shutdown_handler: syn::Lit,
        span: Span,
    ) -> Result<(), syn::Error> {
        if self.shutdown_handler.is_some() {
            return Err(syn::Error::new(
                span,
                "`shutdown_handler` set multiple times.",
            ));
        }
        let shutdown_handler = parse_string(shutdown_handler, span, "shutdown_handler")?;
        self.shutdown_handler = Some(shutdown_handler);
        Ok(())
    }

    fn build(self, span: Span) -> Result<FinalConfig, syn::Error> {
        let services = match self.services {
            None => {
                let msg = "At least one service must be specified in `services`";
                return Err(syn::Error::new(span, msg));
            }
            Some(x) => x,
        };
        Ok(FinalConfig {
            services,
            init_subscriber: self.init_subscriber.unwrap_or_default(),
            shutdown_handler: self.shutdown_handler,
        })
    }
}

#[derive(Default)]
struct Services {
    stream: bool,
    data: bool,
    diagnostics: bool,
    resource: bool,
}

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

fn build_config(input: syn::ItemFn, args: AttributeArgs) -> Result<FinalConfig, syn::Error> {
    if input.sig.asyncness.is_none() {
        let msg = "the `async` keyword is missing from the function declaration";
        return Err(syn::Error::new_spanned(input.sig.fn_token, msg));
    }
    let mut config = Configuration::new();
    for arg in args {
        match arg {
            syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue)) => {
                let ident = namevalue
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&namevalue, "Must have specified ident")
                    })?
                    .to_string()
                    .to_lowercase();
                match ident.as_str() {
                    "init_subscriber" => config.set_init_subscriber(
                        namevalue.lit.clone(),
                        syn::spanned::Spanned::span(&namevalue.lit),
                    )?,
                    "shutdown_handler" => config.set_shutdown_handler(
                        namevalue.lit.clone(),
                        syn::spanned::Spanned::span(&namevalue.lit),
                    )?,
                    name => {
                        let msg = format!(
                            "Unknown attribute {} is specified; expected one of: `services`, `init_subscriber`, `shutdown_handler`",
                            name,
                        );
                        return Err(syn::Error::new_spanned(namevalue, msg));
                    }
                }
            }
            syn::NestedMeta::Meta(syn::Meta::List(list)) => {
                let ident = list
                    .path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&list, "Must have specified ident"))?
                    .to_string()
                    .to_lowercase();
                match ident.as_str() {
                    "services" => config
                        .set_services(&list.nested, syn::spanned::Spanned::span(&list.nested))?,
                    name => {
                        let msg = format!(
                            "Unknown attribute {} is specified; expected one of: `services`, `init_subscriber`, `shutdown_handler`",
                            name,
                        );
                        return Err(syn::Error::new_spanned(list, msg));
                    }
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "Unknown attribute inside the macro",
                ));
            }
        }
    }
    config.build(syn::spanned::Spanned::span(&input))
}

fn parse_string(int: syn::Lit, span: Span, field: &str) -> Result<String, syn::Error> {
    match int {
        syn::Lit::Str(s) => Ok(s.value()),
        syn::Lit::Verbatim(s) => Ok(s.to_string()),
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{}` as string.", field),
        )),
    }
}

fn parse_bool(bool: syn::Lit, span: Span, field: &str) -> Result<bool, syn::Error> {
    match bool {
        syn::Lit::Bool(b) => Ok(b.value),
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{}` as bool.", field),
        )),
    }
}

fn parse_services(list: &AttributeArgs) -> Result<Vec<String>, syn::Error> {
    list.iter()
        .map(|item| match item {
            syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                let svc = path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&list, "Must have specified ident"))?
                    .to_string()
                    .to_lowercase();
                if !["data", "diagnostics", "resource", "stream"].contains(&svc.as_str()) {
                    let msg = format!(
                        "invalid service {}; must be one of `data`, `diagnostics`, `resource`, `stream`",
                        svc,
                    );
                    return Err(syn::Error::new_spanned(path, msg))
                }
                Ok(svc)
            },
            other => {
                let msg = format!(
                    "invalid service specification: must contain one or more of `data`, `diagnostics`, `resource`, `stream`",
                );
                Err(syn::Error::new_spanned(other, msg))
            }
        })
        .collect()
}

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
        plugin = quote! { #plugin.data_service(service.clone()) };
    }
    if config.services.diagnostics {
        plugin = quote! { #plugin.diagnostics_service(service.clone()) };
    }
    if config.services.resource {
        plugin = quote! { #plugin.resource_service(service.clone()) };
    }
    if config.services.stream {
        plugin = quote! { #plugin.stream_service(service.clone()) };
    }
    let init_subscriber = config.init_subscriber;
    if init_subscriber {
        plugin = quote! { #plugin.init_subscriber(#init_subscriber) };
    }
    if let Some(x) = config.shutdown_handler {
        let shutdown_handler = quote! { #x.parse()? };
        plugin = quote! { #plugin.shutdown_handler(#shutdown_handler) };
    }

    let expanded = quote! {
        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let listener = ::grafana_plugin_sdk::backend::initialize().await?;
            let service = #body;
            let plugin = #plugin
                .start(listener)
                .await?;
            Ok(())
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };

    let config = if input.sig.ident != "plugin" {
        let msg = "the plugin function must be named 'plugin'";
        Err(syn::Error::new_spanned(&input.sig.ident, msg))
    } else if !input.sig.inputs.is_empty() {
        let msg = "the plugin function cannot accept arguments";
        Err(syn::Error::new_spanned(&input.sig.inputs, msg))
    } else {
        AttributeArgs::parse_terminated
            .parse(args)
            .and_then(|args| build_config(input.clone(), args))
    };

    match config {
        Ok(c) => parse_knobs(input, c),
        Err(e) => token_stream_with_error(parse_knobs(input, DEFAULT_ERROR_CONFIG), e),
    }
}
