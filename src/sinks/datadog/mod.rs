use crate::{
    http::HttpClient,
    sinks::{HealthcheckError, UriParseError},
};
use http::{Request, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use snafu::ResultExt;
use std::sync::Arc;

pub mod events;
pub mod logs;
pub mod metrics;
pub mod reporter;

type ApiKey = Arc<str>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Us,
    Eu,
}

async fn healthcheck(endpoint: String, api_key: String, client: HttpClient) -> crate::Result<()> {
    let uri = format!("{}/api/v1/validate", endpoint)
        .parse::<Uri>()
        .context(UriParseError)?;

    let request = Request::get(uri)
        .header("DD-API-KEY", api_key)
        .body(hyper::Body::empty())
        .unwrap();

    let response = client.send(request).await?;

    match response.status() {
        StatusCode::OK => Ok(()),
        other => Err(HealthcheckError::UnexpectedStatus { status: other }.into()),
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Clone, Debug)]
#[repr(u8)]
enum CheckStatus {
    Ok = 0,
    Warning = 1,
    Critical = 2,
    Unknown = 3,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct DatadogCheck {
    check: String,
    hostname: String,
    message: String,
    status: CheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

async fn send_check(
    endpoint: String,
    api_key: String,
    client: HttpClient,
    check: DatadogCheck,
) -> crate::Result<()> {
    let uri = format!("{}/api/v1/check_run", endpoint)
        .parse::<Uri>()
        .context(UriParseError)?;

    let request = Request::post(uri)
        .header("DD-API-KEY", api_key)
        .body(
            serde_json::to_string(&check)
                .expect("json encoding should never fail")
                .into(),
        )
        .unwrap();

    let response = client.send(request).await?;

    match response.status() {
        StatusCode::OK => Ok(()),
        other => Err(HealthcheckError::UnexpectedStatus { status: other }.into()),
    }
}
