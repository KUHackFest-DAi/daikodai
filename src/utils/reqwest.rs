use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Deserialize, Serialize)]
struct IpInfoResponse {
    ip: String,
}

pub async fn get_external_ip() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("curl/7.64.1")
        .build()?;

    let response = client.get("https://ipinfo.io").send().await?;

    let body: IpInfoResponse = response.json().await?;
    log::info!("Found external IP: {}", body.ip);

    Ok(body.ip)
}
