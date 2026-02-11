use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use tracing::{debug, info, instrument};

use crate::auth::AgentUser;
use crate::error::{AppError, AppResult};
use crate::id::generate_id;
use crate::models::{CaptureRun, CaptureRunItemWithContext, CaptureStatus};
use crate::repo::{CaptureRepo, CaptureRunRepo, ShaderVersionRepo};
use crate::state::AppState;

#[instrument(skip(s3, content_type), level = "debug")]
async fn generate_presigned_put_url(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    content_type: &str,
    expiry_secs: u64,
) -> AppResult<String> {
    let presigned = s3
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::builder()
                .expires_in(std::time::Duration::from_secs(expiry_secs))
                .build()
                .expect("valid presigning config"),
        )
        .await
        .map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate presigned URL: {}", e))
        })?;
    Ok(presigned.uri().to_string())
}

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub agent_id: Option<String>,
    pub items: Vec<CreateRunItemRequest>,
    pub metadata_json: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunItemRequest {
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
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
    /// World version this capture was taken against (from WorkItem)
    pub world_version_id: String,
    /// Scene version this capture was taken against (from WorkItem)
    pub scene_version_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ClaimItemResponse {
    pub capture_id: String,
    pub presigned_url: String,
    pub image_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmUploadRequest {
    pub image_path: Option<String>,
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
    pub image_url: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_runs).post(create_run))
        .route("/{id}", get(get_run))
        .route("/{id}/items", get(list_run_items))
        .route("/{id}/items/{item_id}/fail", post(fail_item))
        .route("/{id}/items/{item_id}/claim", post(claim_item))
        .route("/{id}/items/{item_id}/confirm-upload", post(confirm_upload))
        .route("/{id}/complete", post(complete_run))
}

#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn create_run(
    _user: AgentUser,
    State(state): State<AppState>,
    Json(request): Json<CreateRunRequest>,
) -> AppResult<(StatusCode, Json<CaptureRun>)> {
    let run_id = generate_id();
    let total_items = request.items.len() as i32;

    let run = CaptureRunRepo::create(
        state.db(),
        &run_id,
        request.agent_id.as_deref(),
        total_items,
        request.metadata_json.as_ref(),
    )
    .await?;

    let owned_items: Vec<_> = request
        .items
        .iter()
        .map(|item| {
            (
                generate_id(),
                run_id.clone(),
                item.shader_version_id.clone(),
                item.scene_id.clone(),
                item.profile.clone(),
            )
        })
        .collect();

    let items: Vec<_> = owned_items
        .iter()
        .map(|(id, rid, sv, sc, p)| {
            (
                id.as_str(),
                rid.as_str(),
                sv.as_str(),
                sc.as_str(),
                p.as_deref(),
            )
        })
        .collect();
    CaptureRunRepo::insert_items(state.db(), &items).await?;

    info!(run_id = %run_id, total_items, "Created capture run");

    state.track(
        "capture_run_started",
        "system",
        serde_json::json!({
            "run_id": run_id,
            "total_items": total_items,
            "agent_id": request.agent_id,
        }),
    );

    Ok((StatusCode::CREATED, Json(run)))
}

#[instrument(skip(state, _user), fields(user_id = _user.user.id))]
async fn list_runs(
    _user: AgentUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<CaptureRun>>> {
    let runs = CaptureRunRepo::list(state.db()).await?;
    Ok(Json(runs))
}

#[instrument(skip(state, _user), fields(user_id = _user.user.id))]
async fn get_run(
    _user: AgentUser,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> AppResult<Json<CaptureRun>> {
    let run = CaptureRunRepo::get_by_id(state.db(), &run_id).await?;
    Ok(Json(run))
}

#[instrument(skip(state, _user), fields(user_id = _user.user.id))]
async fn list_run_items(
    _user: AgentUser,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> AppResult<Json<Vec<CaptureRunItemWithContext>>> {
    let items = CaptureRunRepo::list_items_with_context(state.db(), &run_id).await?;
    Ok(Json(items))
}

#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn fail_item(
    _user: AgentUser,
    State(state): State<AppState>,
    Path((run_id, item_id)): Path<(String, String)>,
    Json(request): Json<FailItemRequest>,
) -> AppResult<StatusCode> {
    let item = CaptureRunRepo::get_item_by_id(state.db(), &item_id).await?;
    if item.run_id.as_ref() != run_id {
        return Err(AppError::NotFound(format!(
            "Run item '{}' not found in run '{}'",
            item_id, run_id
        )));
    }

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

#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn claim_item(
    _user: AgentUser,
    State(state): State<AppState>,
    Path((run_id, item_id)): Path<(String, String)>,
    Json(request): Json<ClaimItemRequest>,
) -> AppResult<Json<ClaimItemResponse>> {
    let db = state.db();

    let item = CaptureRunRepo::get_item_by_id(db, &item_id).await?;
    if item.run_id.as_ref() != run_id {
        return Err(AppError::NotFound(format!(
            "Run item '{}' not found in run '{}'",
            item_id, run_id
        )));
    }

    let shader_id = ShaderVersionRepo::get_shader_id(db, item.shader_version_id.as_ref()).await?;

    let capture_id = generate_id();
    let r2_key = format!(
        "captures/{}/{}/{}.webp",
        shader_id, item.scene_id, capture_id
    );

    let r2_config = &state.config().r2;
    let image_url = r2_config.public_url_for_key(&r2_key);

    let presigned_url = if let Some(s3) = state.s3() {
        let bucket = r2_config.bucket.as_deref().unwrap_or("glint");
        generate_presigned_put_url(s3, bucket, &r2_key, "image/webp", 3600).await?
    } else {
        format!("https://r2.example.com/{}", r2_key)
    };

    let mut tx = state.begin_tx().await?;

    CaptureRepo::insert(
        &mut *tx,
        &capture_id,
        item.shader_version_id.as_ref(),
        item.scene_id.as_ref(),
        item.profile.as_deref(),
        None,
        Some(&image_url),
        Some(request.resolution_width),
        Some(request.resolution_height),
        Some(request.captured_at),
        Some(&request.world_version_id),
        Some(&request.scene_version_id),
        CaptureStatus::Uploading,
    )
    .await?;

    CaptureRunRepo::claim_item(&mut *tx, &item_id, &capture_id).await?;

    tx.commit().await?;

    debug!(run_id, item_id, capture_id = %capture_id, "Claimed run item");
    Ok(Json(ClaimItemResponse {
        capture_id,
        presigned_url,
        image_url,
    }))
}

#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn confirm_upload(
    _user: AgentUser,
    State(state): State<AppState>,
    Path((run_id, item_id)): Path<(String, String)>,
    Json(request): Json<ConfirmUploadRequest>,
) -> AppResult<StatusCode> {
    let item = CaptureRunRepo::get_item_by_id(state.db(), &item_id).await?;
    if item.run_id.as_ref() != run_id {
        return Err(AppError::NotFound(format!(
            "Run item '{}' not found in run '{}'",
            item_id, run_id
        )));
    }

    let capture_id = item
        .capture_id
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Run item has no capture_id".to_string()))?;
    let capture_id_str: &str = capture_id.as_ref();

    let mut tx = state.begin_tx().await?;
    CaptureRepo::confirm_upload(&mut *tx, capture_id_str, request.image_path.as_deref()).await?;
    CaptureRunRepo::complete_item(&mut *tx, &item_id, capture_id_str, None).await?;
    tx.commit().await?;

    // Queue capture metadata processing (fire-and-forget)
    let _ = state.metadata_tx().send(capture_id_str.to_string());

    debug!(item_id, capture_id = capture_id_str, "Upload confirmed");
    Ok(StatusCode::OK)
}

#[instrument(skip(state, _user), fields(user_id = _user.user.id))]
async fn complete_run(
    _user: AgentUser,
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

    state.track(
        "capture_run_completed",
        "system",
        serde_json::json!({
            "run_id": run_id,
            "status": run.status,
            "completed_items": run.completed_items,
            "failed_items": run.failed_items,
        }),
    );

    Ok(Json(run))
}

pub fn failure_router() -> Router<AppState> {
    Router::new().route("/report-failure", post(report_failure))
}

#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn report_failure(
    _user: AgentUser,
    State(state): State<AppState>,
    Json(request): Json<ReportFailureRequest>,
) -> AppResult<StatusCode> {
    ShaderVersionRepo::increment_failure_count(
        state.db(),
        &request.shader_version_id,
        &request.error_message,
    )
    .await?;

    debug!(
        shader_version_id = %request.shader_version_id,
        "Reported shader capture failure"
    );

    Ok(StatusCode::OK)
}

pub fn upload_router() -> Router<AppState> {
    Router::new().route("/upload-url", post(get_upload_url))
}

#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn get_upload_url(
    _user: AgentUser,
    State(state): State<AppState>,
    Json(request): Json<UploadUrlRequest>,
) -> AppResult<Json<UploadUrlResponse>> {
    let capture_id = generate_id();
    let r2_key = format!(
        "captures/{}/{}/{}.webp",
        request.shader_id, request.scene_id, capture_id
    );

    let r2_config = &state.config().r2;
    let image_url = r2_config.public_url_for_key(&r2_key);

    let presigned_url = if let Some(s3) = state.s3() {
        let bucket = r2_config.bucket.as_deref().unwrap_or("glint");
        generate_presigned_put_url(s3, bucket, &r2_key, "image/webp", 3600).await?
    } else {
        format!("https://r2.example.com/{}", r2_key)
    };

    Ok(Json(UploadUrlResponse {
        capture_id,
        r2_key,
        presigned_url,
        image_url,
    }))
}
