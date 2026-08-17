use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

#[derive(Clone)]
pub struct ControllerCliClient {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl ControllerCliClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            base_url,
            api_key,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        if !self.api_key.is_empty() {
            let auth_val = format!("Bearer {}", self.api_key);
            let mut auth_header = HeaderValue::from_str(&auth_val)?;
            auth_header.set_sensitive(true);
            headers.insert(AUTHORIZATION, auth_header);
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await
            .context(format!("Failed to connect to {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("API request failed ({}): {}", status, body);
        }

        let data: T = resp.json().await?;
        Ok(data)
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers()?)
            .json(body)
            .send()
            .await
            .context(format!("Failed to connect to {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("API request failed ({}): {}", status, body);
        }

        let data: T = resp.json().await?;
        Ok(data)
    }

    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .delete(&url)
            .headers(self.headers()?)
            .send()
            .await
            .context(format!("Failed to connect to {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("API request failed ({}): {}", status, body);
        }

        let data: T = resp.json().await?;
        Ok(data)
    }
}
