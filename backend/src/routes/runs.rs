use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use nanoid::nanoid;
use serde::Deserialize;
use tracing::{debug, info};

use crate::error::{AppError, AppResult};
use crate::models::{CaptureRun, CaptureRunItem, CaptureRunItemWithContext};
use crate::repo::{CaptureRepo, CaptureRunRepo};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub agent_id: Option<String>,
    pub items: Vec<CreateRunItemRequest>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunItemRequest {
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteItemRequest {
    pub capture_id: String,
    pub screenshot_path: String,
    pub screenshot_url: String,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FailItemRequest {
    pub error_message: String,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReportFailureRequest {
    pub shader_version_id: String,
    pub error_message: String,
}

#[derive(Debug, Deserialize)]
pub struct ClaimItemRequest {
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub captured_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct ClaimItemResponse {
    pub capture_id: String,
    pub presigned_url: String,
    pub screenshot_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmUploadRequest {
    pub screenshot_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadUrlRequest {
    pub shader_id: String,
    pub scene_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct UploadUrlResponse {
    pub capture_id: String,
    pub r2_key: String,
    pub presigned_url: String,
    pub screenshot_url: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_runs).post(create_run))
        .route("/{id}", get(get_run))
        .route("/{id}/items", get(list_run_items))
        .route("/{id}/items/{item_id}/complete", post(complete_item))
        .route("/{id}/items/{item_id}/fail", post(fail_item))
        .route("/{id}/items/{item_id}/claim", post(claim_item))
        .route("/{id}/items/{item_id}/confirm-upload", post(confirm_upload))
        .route("/{id}/complete", post(complete_run))
}

async fn create_run(
    State(state): State<AppState>,
    Json(request): Json<CreateRunRequest>,
) -> AppResult<(StatusCode, Json<CaptureRun>)> {
    let run_id = nanoid!();
    let total_items = request.items.len() as i32;

    let run = CaptureRunRepo::create(
        state.db(),
        &run_id,
        request.agent_id.as_deref(),
        total_items,
        request.metadata_json.as_deref(),
    )
    .await?;

    let items: Vec<_> = request
        .items
        .iter()
        .map(|item| {
            (
                nanoid!(),
                run_id.clone(),
                item.shader_version_id.clone(),
                item.scene_id.clone(),
                item.profile.clone(),
            )
        })
        .collect();

    CaptureRunRepo::insert_items(state.db(), &items).await?;

    info!(run_id = %run_id, total_items, "Created capture run");
    Ok((StatusCode::CREATED, Json(run)))
}

async fn list_runs(State(state): State<AppState>) -> AppResult<Json<Vec<CaptureRun>>> {
    let runs = CaptureRunRepo::list(state.db()).await?;
    Ok(Json(runs))
}

async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> AppResult<Json<CaptureRun>> {
    let run = CaptureRunRepo::get_by_id(state.db(), &run_id).await?;
    Ok(Json(run))
}

async fn list_run_items(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> AppResult<Json<Vec<CaptureRunItemWithContext>>> {
    let items = CaptureRunRepo::list_items_with_context(state.db(), &run_id).await?;
    Ok(Json(items))
}

async fn complete_item(
    State(state): State<AppState>,
    Path((run_id, item_id)): Path<(String, String)>,
    Json(request): Json<CompleteItemRequest>,
) -> AppResult<StatusCode> {
    let db = state.db();

    let items = CaptureRunRepo::list_items(db, &run_id).await?;
    let item = items
        .iter()
        .find(|i| i.id == item_id)
        .ok_or_else(|| AppError::NotFound(format!("Run item '{}' not found", item_id)))?;

    CaptureRepo::insert(
        db,
        &request.capture_id,
        &item.shader_version_id,
        &item.scene_id,
        item.profile.as_deref(),
        Some(&request.screenshot_path),
        Some(&request.screenshot_url),
        Some(request.resolution_width),
        Some(request.resolution_height),
        Some(request.captured_at),
    )
    .await?;

    CaptureRunRepo::complete_item(db, &item_id, &request.capture_id, request.duration_ms).await?;

    debug!(run_id, item_id, "Completed run item");
    Ok(StatusCode::OK)
}

async fn fail_item(
    State(state): State<AppState>,
    Path((_run_id, item_id)): Path<(String, String)>,
    Json(request): Json<FailItemRequest>,
) -> AppResult<StatusCode> {
    CaptureRunRepo::fail_item(
        state.db(),
        &item_id,
        &request.error_message,
        request.error_log.as_deref(),
        request.duration_ms,
    )
    .await?;

    Ok(StatusCode::OK)
}

async fn claim_item(
    State(state): State<AppState>,
    Path((run_id, item_id)): Path<(String, String)>,
    Json(request): Json<ClaimItemRequest>,
) -> AppResult<Json<ClaimItemResponse>> {
    let db = state.db();

    let items = CaptureRunRepo::list_items(db, &run_id).await?;
    let item = items
        .iter()
        .find(|i| i.id == item_id)
        .ok_or_else(|| AppError::NotFound(format!("Run item '{}' not found", item_id)))?;

    // Look up shader_id from shader_version
    struct ShaderIdRow {
        shader_id: String,
    }
    let shader_row = sqlx::query_as!(
        ShaderIdRow,
        "SELECT shader_id FROM shader_versions WHERE id = $1",
        item.shader_version_id
    )
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let capture_id = nanoid!();
    let r2_key = format!(
        "captures/{}/{}/{}.png",
        shader_row.shader_id, item.scene_id, capture_id
    );

    let r2_config = &state.config().r2;
    let screenshot_url = r2_config.public_url_for_key(&r2_key);

    // Generate presigned URL
    let presigned_url = if let Some(s3) = state.s3() {
        let bucket = r2_config.bucket.as_deref().unwrap_or("glint");
        let presigned = s3
            .put_object()
            .bucket(bucket)
            .key(&r2_key)
            .content_type("image/png")
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::builder()
                    .expires_in(std::time::Duration::from_secs(3600))
                    .build()
                    .expect("valid presigning config"),
            )
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to generate presigned URL: {}", e))
            })?;
        presigned.uri().to_string()
    } else {
        format!("https://r2.example.com/{}", r2_key)
    };

    // Insert capture in 'uploading' status
    CaptureRepo::insert_uploading(
        db,
        &capture_id,
        &item.shader_version_id,
        &item.scene_id,
        item.profile.as_deref(),
        Some(&screenshot_url),
        Some(request.resolution_width),
        Some(request.resolution_height),
        Some(request.captured_at),
    )
    .await?;

    // Update run item status to 'running' with capture_id
    sqlx::query!(
        "UPDATE capture_run_items SET status = 'running', capture_id = $2, started_at = now() WHERE id = $1",
        item_id,
        capture_id,
    )
    .execute(db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    debug!(run_id, item_id, capture_id = %capture_id, "Claimed run item");
    Ok(Json(ClaimItemResponse {
        capture_id,
        presigned_url,
        screenshot_url,
    }))
}

async fn confirm_upload(
    State(state): State<AppState>,
    Path((_run_id, item_id)): Path<(String, String)>,
    Json(request): Json<ConfirmUploadRequest>,
) -> AppResult<StatusCode> {
    let db = state.db();

    // Look up the run item to get capture_id
    let item = sqlx::query_as!(
        CaptureRunItem,
        "SELECT * FROM capture_run_items WHERE id = $1",
        item_id
    )
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or_else(|| AppError::NotFound(format!("Run item '{}' not found", item_id)))?;

    let capture_id = item
        .capture_id
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Run item has no capture_id".to_string()))?;

    CaptureRepo::confirm_upload(db, capture_id, request.screenshot_path.as_deref()).await?;
    CaptureRunRepo::complete_item(db, &item_id, capture_id, None).await?;

    debug!(item_id, capture_id, "Upload confirmed");
    Ok(StatusCode::OK)
}

async fn complete_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> AppResult<Json<CaptureRun>> {
    let run = CaptureRunRepo::complete(state.db(), &run_id).await?;

    info!(
        run_id = %run_id,
        status = %run.status,
        completed = run.completed_items,
        failed = run.failed_items,
        "Capture run finalized"
    );

    Ok(Json(run))
}

pub fn failure_router() -> Router<AppState> {
    Router::new().route("/report-failure", post(report_failure))
}

async fn report_failure(
    State(state): State<AppState>,
    Json(request): Json<ReportFailureRequest>,
) -> AppResult<StatusCode> {
    sqlx::query!(
        r#"
        UPDATE shader_versions
        SET capture_failure_count = capture_failure_count + 1,
            last_capture_error = $2
        WHERE id = $1
        "#,
        request.shader_version_id,
        request.error_message,
    )
    .execute(state.db())
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    debug!(
        shader_version_id = %request.shader_version_id,
        "Reported shader capture failure"
    );

    Ok(StatusCode::OK)
}

pub fn upload_router() -> Router<AppState> {
    Router::new().route("/upload-url", post(get_upload_url))
}

async fn get_upload_url(
    State(state): State<AppState>,
    Json(request): Json<UploadUrlRequest>,
) -> AppResult<Json<UploadUrlResponse>> {
    let capture_id = nanoid!();
    let r2_key = format!(
        "captures/{}/{}/{}.png",
        request.shader_id, request.scene_id, capture_id
    );

    let r2_config = &state.config().r2;
    let screenshot_url = r2_config.public_url_for_key(&r2_key);

    let Some(s3) = state.s3() else {
        let presigned_url = format!("https://r2.example.com/{}", r2_key);
        return Ok(Json(UploadUrlResponse {
            capture_id,
            r2_key,
            presigned_url,
            screenshot_url,
        }));
    };

    let bucket = r2_config.bucket.as_deref().unwrap_or("glint");
    let presigned = s3
        .put_object()
        .bucket(bucket)
        .key(&r2_key)
        .content_type("image/png")
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::builder()
                .expires_in(std::time::Duration::from_secs(3600))
                .build()
                .expect("valid presigning config"),
        )
        .await
        .map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate presigned URL: {}", e))
        })?;

    Ok(Json(UploadUrlResponse {
        capture_id,
        r2_key,
        presigned_url: presigned.uri().to_string(),
        screenshot_url,
    }))
}
