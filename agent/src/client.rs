//! HTTP client for backend API communication

use anyhow::{Context, Result};
use glint_shared::{
    CompleteJobRequest, FailJobRequest, JobPayload, PrepareUploadRequest, PrepareUploadResponse,
};
use reqwest::Client;
use tracing::{debug, instrument};

/// API client for the Glint backend
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Claim the next available job
    #[instrument(skip(self))]
    pub async fn claim_job(&self) -> Result<Option<JobPayload>> {
        let url = format!("{}/api/agent/jobs/next", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to send job claim request")?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            debug!("No jobs available");
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "(failed to read response body)".to_string());
            anyhow::bail!("Failed to claim job: {} - {}", status, body);
        }

        let job: Option<JobPayload> = response
            .json()
            .await
            .context("Failed to parse job payload")?;

        if let Some(ref job) = job {
            debug!(job_id = %job.id, "Claimed job");
        } else {
            debug!("No jobs available");
        }

        Ok(job)
    }

    /// Send heartbeat for a job
    #[instrument(skip(self))]
    pub async fn heartbeat(&self, job_id: &str) -> Result<()> {
        let url = format!("{}/api/agent/jobs/{}/heartbeat", self.base_url, job_id);

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to send heartbeat")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "(failed to read response body)".to_string());
            anyhow::bail!("Heartbeat failed: {} - {}", status, body);
        }

        debug!(job_id, "Heartbeat sent");
        Ok(())
    }

    /// Request pre-signed upload URLs
    #[instrument(skip(self))]
    pub async fn prepare_upload(
        &self,
        job_id: &str,
        files: Vec<String>,
    ) -> Result<PrepareUploadResponse> {
        let url = format!("{}/api/agent/jobs/{}/prepare", self.base_url, job_id);

        let request = PrepareUploadRequest { files };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to request upload URLs")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "(failed to read response body)".to_string());
            anyhow::bail!("Failed to get upload URLs: {} - {}", status, body);
        }

        response
            .json()
            .await
            .context("Failed to parse upload URLs response")
    }

    /// Report job completion
    #[instrument(skip(self, request))]
    pub async fn complete_job(&self, job_id: &str, request: CompleteJobRequest) -> Result<()> {
        let url = format!("{}/api/agent/jobs/{}/complete", self.base_url, job_id);

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send completion request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "(failed to read response body)".to_string());
            anyhow::bail!("Failed to complete job: {} - {}", status, body);
        }

        debug!(job_id, "Job completed");
        Ok(())
    }

    /// Report job failure
    #[instrument(skip(self))]
    pub async fn fail_job(&self, job_id: &str, error_message: &str) -> Result<()> {
        let url = format!("{}/api/agent/jobs/{}/fail", self.base_url, job_id);

        let request = FailJobRequest {
            error_message: error_message.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send failure request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "(failed to read response body)".to_string());
            anyhow::bail!("Failed to report job failure: {} - {}", status, body);
        }

        debug!(job_id, "Job failure reported");
        Ok(())
    }
}
