#[allow(clippy::all, clippy::nursery, clippy::pedantic)]
pub mod pluginv2 {
    tonic::include_proto!("pluginv2");
}

pub mod backend;
pub mod data;
