#![allow(dead_code, unused_variables)]

mod a {
    use grafana_plugin_sdk::prelude::*;

    #[derive(Clone, GrafanaPlugin)]
    #[grafana_plugin(plugin_type = "datasource")]
    struct MyPlugin;

    #[grafana_plugin_sdk::main(services(data))]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}

fn main() {}
