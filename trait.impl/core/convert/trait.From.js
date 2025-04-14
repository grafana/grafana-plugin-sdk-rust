(function() {
    var implementors = Object.fromEntries([["grafana_plugin_sdk",[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.ConvertFromError.html\" title=\"enum grafana_plugin_sdk::backend::ConvertFromError\">ConvertFromError</a>&gt; for <a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.Error.html\" title=\"enum grafana_plugin_sdk::backend::Error\">Error</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.ConvertFromError.html\" title=\"enum grafana_plugin_sdk::backend::ConvertFromError\">ConvertFromError</a>&gt; for Status"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.ConvertToError.html\" title=\"enum grafana_plugin_sdk::backend::ConvertToError\">ConvertToError</a>&gt; for <a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.Error.html\" title=\"enum grafana_plugin_sdk::backend::Error\">Error</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.HealthStatus.html\" title=\"enum grafana_plugin_sdk::backend::HealthStatus\">HealthStatus</a>&gt; for <a class=\"enum\" href=\"grafana_plugin_sdk/pluginv2/check_health_response/enum.HealthStatus.html\" title=\"enum grafana_plugin_sdk::pluginv2::check_health_response::HealthStatus\">HealthStatus</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.PublishStreamStatus.html\" title=\"enum grafana_plugin_sdk::backend::PublishStreamStatus\">PublishStreamStatus</a>&gt; for <a class=\"enum\" href=\"grafana_plugin_sdk/pluginv2/publish_stream_response/enum.Status.html\" title=\"enum grafana_plugin_sdk::pluginv2::publish_stream_response::Status\">Status</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.SubscribeStreamStatus.html\" title=\"enum grafana_plugin_sdk::backend::SubscribeStreamStatus\">SubscribeStreamStatus</a>&gt; for <a class=\"enum\" href=\"grafana_plugin_sdk/pluginv2/subscribe_stream_response/enum.Status.html\" title=\"enum grafana_plugin_sdk::pluginv2::subscribe_stream_response::Status\">Status</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/data/enum.TypeInfoType.html\" title=\"enum grafana_plugin_sdk::data::TypeInfoType\">TypeInfoType</a>&gt; for DataType"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/live/enum.ChannelError.html\" title=\"enum grafana_plugin_sdk::live::ChannelError\">Error</a>&gt; for <a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.ConvertFromError.html\" title=\"enum grafana_plugin_sdk::backend::ConvertFromError\">ConvertFromError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/pluginv2/admission_request/enum.Operation.html\" title=\"enum grafana_plugin_sdk::pluginv2::admission_request::Operation\">Operation</a>&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.i32.html\">i32</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/pluginv2/check_health_response/enum.HealthStatus.html\" title=\"enum grafana_plugin_sdk::pluginv2::check_health_response::HealthStatus\">HealthStatus</a>&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.i32.html\">i32</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/pluginv2/publish_stream_response/enum.Status.html\" title=\"enum grafana_plugin_sdk::pluginv2::publish_stream_response::Status\">Status</a>&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.i32.html\">i32</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"grafana_plugin_sdk/pluginv2/subscribe_stream_response/enum.Status.html\" title=\"enum grafana_plugin_sdk::pluginv2::subscribe_stream_response::Status\">Status</a>&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.i32.html\">i32</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"grafana_plugin_sdk/backend/struct.CheckHealthResponse.html\" title=\"struct grafana_plugin_sdk::backend::CheckHealthResponse\">CheckHealthResponse</a>&gt; for <a class=\"struct\" href=\"grafana_plugin_sdk/pluginv2/struct.CheckHealthResponse.html\" title=\"struct grafana_plugin_sdk::pluginv2::CheckHealthResponse\">CheckHealthResponse</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"grafana_plugin_sdk/backend/struct.CollectMetricsResponse.html\" title=\"struct grafana_plugin_sdk::backend::CollectMetricsResponse\">CollectMetricsResponse</a>&gt; for <a class=\"struct\" href=\"grafana_plugin_sdk/pluginv2/struct.CollectMetricsResponse.html\" title=\"struct grafana_plugin_sdk::pluginv2::CollectMetricsResponse\">CollectMetricsResponse</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"grafana_plugin_sdk/backend/struct.MetricsPayload.html\" title=\"struct grafana_plugin_sdk::backend::MetricsPayload\">Payload</a>&gt; for <a class=\"struct\" href=\"grafana_plugin_sdk/pluginv2/collect_metrics_response/struct.Payload.html\" title=\"struct grafana_plugin_sdk::pluginv2::collect_metrics_response::Payload\">Payload</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"grafana_plugin_sdk/backend/struct.SubscribeStreamResponse.html\" title=\"struct grafana_plugin_sdk::backend::SubscribeStreamResponse\">SubscribeStreamResponse</a>&gt; for <a class=\"struct\" href=\"grafana_plugin_sdk/pluginv2/struct.SubscribeStreamResponse.html\" title=\"struct grafana_plugin_sdk::pluginv2::SubscribeStreamResponse\">SubscribeStreamResponse</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"grafana_plugin_sdk/pluginv2/struct.TimeRange.html\" title=\"struct grafana_plugin_sdk::pluginv2::TimeRange\">TimeRange</a>&gt; for <a class=\"struct\" href=\"grafana_plugin_sdk/backend/struct.TimeRange.html\" title=\"struct grafana_plugin_sdk::backend::TimeRange\">TimeRange</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;Error&gt; for <a class=\"enum\" href=\"grafana_plugin_sdk/backend/enum.Error.html\" title=\"enum grafana_plugin_sdk::backend::Error\">Error</a>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[8148]}