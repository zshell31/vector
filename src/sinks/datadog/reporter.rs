use super::{metrics::DatadogConfig, send_check, CheckStatus, DatadogCheck};
use crate::{
    config::{DataType, SinkConfig, SinkContext, SinkDescription},
    http::HttpClient,
    sinks::{Healthcheck, VectorSink},
};
use futures::StreamExt;
use http::uri::InvalidUri;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;

#[derive(Debug, Snafu)]
enum BuildError {
    #[snafu(display("Invalid host {:?}: {:?}", host, source))]
    InvalidHost { host: String, source: InvalidUri },
}

#[derive(Clone)]
struct DatadogState {
    last_sent_timestamp: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct DatadogReporterConfig {
    #[serde(flatten)]
    pub datadog_config: DatadogConfig,
}

inventory::submit! {
    SinkDescription::new::<DatadogReporterConfig>("datadog_reporter")
}

impl_generate_config_from_default!(DatadogReporterConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "datadog_reporter")]
impl SinkConfig for DatadogReporterConfig {
    async fn build(&self, cx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        let client = HttpClient::new(None)?;
        spawn_periodic_check(
            self.datadog_config.get_api_endpoint(),
            self.datadog_config.api_key.clone(),
            client,
        );
        self.datadog_config.build(cx).await
    }

    fn input_type(&self) -> DataType {
        DataType::Metric
    }

    fn sink_type(&self) -> &'static str {
        "datadog_reporter"
    }
}

pub fn spawn_periodic_check(endpoint: String, api_key: String, client: HttpClient) {
    let interval = IntervalStream::new(tokio::time::interval(Duration::from_secs(60)));
    let task = interval.for_each(move |_| {
        let api_key = api_key.clone();
        let endpoint = endpoint.clone();
        let client = client.clone();
        let check = DatadogCheck {
            check: "vector.up".to_string(),
            hostname: "prognant-dev-laptop".to_string(),
            message: "vector running".to_string(),
            status: CheckStatus::Ok,
            timestamp: None,
            tags: None,
        };
        async move {
            debug!("Sending periodic check to Datadog intake.");
            if let Err(error) = send_check(endpoint, api_key, client, check).await {
                error!(
                    message = "Failed to send check to Datadog.",
                    %error
                );
            }
        }
    });
    tokio::spawn(task);
}
