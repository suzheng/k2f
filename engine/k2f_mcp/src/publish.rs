use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct PublishResponse {
    #[serde(rename = "appearanceHash")]
    pub appearance_hash: String,
    pub url: String,
    pub iframe: String,
}

pub async fn publish_bytes(origin: &str, bytes: &[u8]) -> Result<PublishResponse, String> {
    let base = origin.trim_end_matches('/');
    let url = format!("{base}/api/publish");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("publish client: {e}"))?;
    let resp = client
        .post(&url)
        .header("content-type", "application/zip")
        .body(bytes.to_vec())
        .send()
        .await
        .map_err(|e| format!("publish request failed: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("publish HTTP {status}: {body}"));
    }
    resp.json::<PublishResponse>()
        .await
        .map_err(|e| format!("publish response json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn publish_posts_zip_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/publish"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "appearanceHash": "abc123",
                "url": "/v/abc123",
                "iframe": "<iframe src=\"/v/abc123\"></iframe>"
            })))
            .mount(&server)
            .await;

        let bytes = b"PK\x03\x04fake";
        let out = publish_bytes(&server.uri(), bytes).await.unwrap();
        assert_eq!(out.appearance_hash, "abc123");
        assert_eq!(out.url, "/v/abc123");
    }
}
