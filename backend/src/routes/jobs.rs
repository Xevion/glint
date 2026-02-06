use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, put},
};

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{CreateJobRequest, Job},
    repo::{JobRepo, ShaderVersionRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs).post(create_job))
        .route("/{id}", delete(delete_job))
        .route("/{id}/cancel", put(cancel_job))
        .route("/{id}/retry", put(retry_job))
        .route("/{id}/release", put(release_job))
}

/// GET /api/jobs - List all jobs with details (admin)
async fn list_jobs(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<crate::repo::job::JobWithDetails>>> {
    let jobs = JobRepo::list_with_details(state.db()).await?;
    Ok(Json(jobs))
}

/// POST /api/jobs - Create a new job (admin)
async fn create_job(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<CreateJobRequest>,
) -> AppResult<(StatusCode, Json<Job>)> {
    // Verify shader version exists
    if !ShaderVersionRepo::exists_by_id(state.db(), &request.shader_version_id).await? {
        return Err(AppError::NotFound("Shader version not found".into()));
    }

    let id = nanoid::nanoid!();
    let job = JobRepo::create(state.db(), &id, &request).await?;

    Ok((StatusCode::CREATED, Json(job)))
}

/// DELETE /api/jobs/{id} - Delete a job (admin)
async fn delete_job(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = JobRepo::delete(state.db(), &job_id).await?;

    if !deleted {
        return Err(AppError::NotFound("Job not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/jobs/{id}/cancel - Cancel a job (admin)
async fn cancel_job(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    let cancelled = JobRepo::cancel(state.db(), &job_id).await?;

    if !cancelled {
        return Err(AppError::Conflict(
            "Job not found or cannot be cancelled (only pending/claimed jobs can be cancelled)"
                .into(),
        ));
    }

    let job = JobRepo::get_by_id(state.db(), &job_id).await?;
    Ok(Json(job))
}

/// PUT /api/jobs/{id}/retry - Retry a failed job (admin)
async fn retry_job(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    let retried = JobRepo::retry(state.db(), &job_id).await?;

    if !retried {
        return Err(AppError::Conflict(
            "Job not found or cannot be retried (only failed jobs can be retried)".into(),
        ));
    }

    let job = JobRepo::get_by_id(state.db(), &job_id).await?;
    Ok(Json(job))
}

/// PUT /api/jobs/{id}/release - Release a claimed job (admin)
async fn release_job(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    let released = JobRepo::release(state.db(), &job_id).await?;

    if !released {
        return Err(AppError::Conflict(
            "Job not found or cannot be released (only claimed/running jobs can be released)"
                .into(),
        ));
    }

    let job = JobRepo::get_by_id(state.db(), &job_id).await?;
    Ok(Json(job))
}
