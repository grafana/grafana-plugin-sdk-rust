wit_bindgen::generate!({
    world: "backend",
    path: "vendor/wit",
    async: true,
    generate_all,
});

pub use exports::*;
