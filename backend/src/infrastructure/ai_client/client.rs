use crate::shared::{AppError, AppResult};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::log::debug;
use tracing::warn;

/// 内部AI服务客户端
#[derive(Clone)]
pub struct AiClient {
    pub base_url: String,
    pub token: Option<String>,
    pub http: reqwest::Client,
}

impl AiClient {
    pub fn new(base_url: String, token: Option<String>) -> AppResult<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Err(AppError::Internal(
                "ai-service base_url is empty".to_string(),
            ));
        }

        if token.is_none() {
            warn!(
                "AI_SERVICE_TOKEN not configured; \
                 requests to ai-service will be sent without Authorization header"
            );
        }

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AppError::Internal(format!("failed to build ai-service http client: {e}"))
            })?;

        Ok(Self {
            base_url,
            token,
            http,
        })
    }

    pub async fn post_internal<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: &Req,
    ) -> AppResult<Resp> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_end_matches('/').trim_start_matches('/')
        );

        let mut req = self.http.post(&url).json(body);

        match &self.token {
            Some(token) => req = req.header("X-Internal-Token", token),

            None => {
                debug!(
                    "AI_SERVICE_TOKEN not configured, sending X-Internal-Token empty for {}",
                    path
                );
            }
        }

        let resp = req.send().await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            let truncated: String = text.chars().take(200).collect();
            return Err(AppError::Upstream(format!(
                "ai-service POST {} failed: {} — {}",
                path, status, truncated
            )));
        }

        let resp: Resp = resp.json().await?;
        Ok(resp)
    }
}
