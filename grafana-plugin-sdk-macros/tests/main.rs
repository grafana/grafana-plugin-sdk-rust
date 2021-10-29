mod a {
    #[derive(Clone)]
    struct MyPlugin;

    #[grafana_plugin_sdk::main(services(data))]
    async fn plugin() -> MyPlugin {
        MyPlugin
    }
}
