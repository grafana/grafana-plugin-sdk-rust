wit_bindgen::generate!({
    world: "backend",
    path: "vendor/wit",
    async: true,
    generate_all,
    debug: true,
});

pub use exports::grafana::plugins::*;
