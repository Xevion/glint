use axum::{Router, body::Bytes, http::StatusCode, routing::post};
use serde::Deserialize;

/// CSP violation report payload sent by browsers.
///
/// Browsers POST with `Content-Type: application/csp-report`, not
/// `application/json`, so we extract raw bytes and deserialize manually.
#[derive(Deserialize)]
struct CspReportEnvelope {
    #[serde(rename = "csp-report")]
    csp_report: CspReport,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
struct CspReport {
    document_uri: Option<String>,
    violated_directive: Option<String>,
    effective_directive: Option<String>,
    blocked_uri: Option<String>,
    status_code: Option<u16>,
    source_file: Option<String>,
    line_number: Option<u32>,
    column_number: Option<u32>,
    original_policy: Option<String>,
}

async fn receive_report(body: Bytes) -> StatusCode {
    let envelope: CspReportEnvelope = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::debug!(error = %e, "Malformed CSP report");
            return StatusCode::BAD_REQUEST;
        }
    };

    let r = &envelope.csp_report;
    tracing::warn!(
        violated = r.violated_directive.as_deref().unwrap_or("unknown"),
        blocked = r.blocked_uri.as_deref().unwrap_or("unknown"),
        document = r.document_uri.as_deref().unwrap_or("unknown"),
        source = r.source_file.as_deref().unwrap_or("unknown"),
        line = r.line_number.unwrap_or(0),
        "CSP violation"
    );
    StatusCode::NO_CONTENT
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new().route("/", post(receive_report))
}
