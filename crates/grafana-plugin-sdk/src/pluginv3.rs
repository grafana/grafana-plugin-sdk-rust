wit_bindgen::generate!({
    world: "backend",
    path: "vendor/wit",
    generate_all,
    pub_export_macro: true,
});

pub use exports::grafana::plugins::*;
