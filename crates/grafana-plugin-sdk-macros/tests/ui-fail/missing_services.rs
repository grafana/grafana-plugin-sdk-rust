#![allow(dead_code, unused_variables)]

mod a {
    #[derive(Clone)]
    struct MyPlugin;

    #[grafana_plugin_sdk::main]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}

fn main() {}
