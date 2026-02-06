use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateJobRequest, Job};

pub struct JobRepo;

/// Row returned when claiming a job
pub struct ClaimedJobRow {
    pub id: String,
    pub shader_version_id: String,
    pub scene_ids: Option<String>,
    pub profiles: Option<String>,
    pub priority: i32,
}

/// Row for job with shader details (admin list)
#[derive(sqlx::FromRow, Serialize)]
pub struct JobWithDetails {
    pub id: String,
    pub shader_version_id: String,
    pub scene_ids: Option<String>,
    pub profiles: Option<String>,
    pub priority: i32,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub agent_id: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_version: String,
    pub scene_count: i32,
}

impl JobRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: &str) -> AppResult<Option<Job>> {
        sqlx::query_as!(Job, "SELECT * FROM jobs WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find job '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: &str) -> AppResult<Job> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Job '{}' not found", id)))
    }

    /// List all jobs with shader details for admin view
    #[instrument(skip(db), level = "debug")]
    pub async fn list_with_details(db: &DbPool) -> AppResult<Vec<JobWithDetails>> {
        let jobs = sqlx::query_as!(
            JobWithDetails,
            r#"
            SELECT
                j.id, j.shader_version_id, j.scene_ids, j.profiles, j.priority,
                j.status, j.attempts, j.max_attempts, j.agent_id, j.claimed_at,
                j.last_heartbeat, j.started_at, j.completed_at, j.error_message,
                j.created_at,
                s.name as shader_name,
                s.slug as shader_slug,
                sv.version as shader_version,
                COALESCE(json_array_length(j.scene_ids::json), 0)::int4 as "scene_count!"
            FROM jobs j
            JOIN shader_versions sv ON sv.id = j.shader_version_id
            JOIN shaders s ON s.id = sv.shader_id
            ORDER BY j.created_at DESC
            "#,
        )
        .fetch_all(db)
        .await
        .context("failed to list jobs with details")?;

        debug!(count = jobs.len(), "Listed jobs");
        Ok(jobs)
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn create(db: &DbPool, id: &str, req: &CreateJobRequest) -> AppResult<Job> {
        let scene_ids_json =
            serde_json::to_string(&req.scene_ids).context("failed to serialize scene_ids")?;
        let profiles_json = req
            .profiles
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .context("failed to serialize profiles")?;
        let priority = req.priority.unwrap_or(0);

        sqlx::query!(
            r#"
            INSERT INTO jobs (id, shader_version_id, scene_ids, profiles, priority, status, attempts, max_attempts, created_at)
            VALUES ($1, $2, $3, $4, $5, 'pending', 0, 3, now())
            "#,
            id,
            req.shader_version_id,
            scene_ids_json,
            profiles_json,
            priority
        )
        .execute(db)
        .await
        .context("failed to create job")?;

        Self::get_by_id(db, id).await
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn delete(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM jobs WHERE id = $1", id)
            .execute(db)
            .await
            .context(format!("failed to delete job '{}'", id))?;
        Ok(result.rows_affected() > 0)
    }

    /// Claim the next available job for an agent
    #[instrument(skip(db), level = "debug")]
    pub async fn claim_next(db: &DbPool, agent_id: &str) -> AppResult<Option<ClaimedJobRow>> {
        let job = sqlx::query_as!(
            ClaimedJobRow,
            r#"
            UPDATE jobs
            SET status = 'claimed',
                agent_id = $1,
                claimed_at = now(),
                last_heartbeat = now(),
                attempts = attempts + 1
            WHERE id = (
                SELECT id FROM jobs
                WHERE status = 'pending'
                  AND attempts < max_attempts
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
            )
            RETURNING id, shader_version_id, scene_ids, profiles, priority
            "#,
            agent_id
        )
        .fetch_optional(db)
        .await
        .context("failed to claim next job")?;

        if job.is_some() {
            debug!(agent_id, "Claimed job");
        }
        Ok(job)
    }

    /// Update heartbeat for a running job, transitioning from claimed to running if needed
    #[instrument(skip(db), level = "trace")]
    pub async fn heartbeat(db: &DbPool, job_id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE jobs
            SET last_heartbeat = now(),
                status = CASE WHEN status = 'claimed' THEN 'running' ELSE status END
            WHERE id = $1 AND status IN ('claimed', 'running')
            "#,
            job_id
        )
        .execute(db)
        .await
        .context(format!("failed to update heartbeat for job '{}'", job_id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Get shader_version_id for an active (claimed or running) job
    #[instrument(skip(db), level = "debug")]
    pub async fn get_active_shader_version_id(
        db: &DbPool,
        job_id: &str,
    ) -> AppResult<Option<String>> {
        sqlx::query_scalar!(
            "SELECT shader_version_id FROM jobs WHERE id = $1 AND status IN ('claimed', 'running')",
            job_id
        )
        .fetch_optional(db)
        .await
        .context(format!(
            "failed to get shader version id for active job '{}'",
            job_id
        ))
        .map_err(Into::into)
    }

    /// Mark job as completed
    #[instrument(skip(db), level = "debug")]
    pub async fn complete(db: &DbPool, job_id: &str) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = 'completed', completed_at = now()
            WHERE id = $1
            "#,
            job_id
        )
        .execute(db)
        .await
        .context(format!("failed to complete job '{}'", job_id))?;

        debug!(job_id, "Job completed");
        Ok(())
    }

    /// Mark job as failed with error message
    #[instrument(skip(db), level = "debug")]
    pub async fn fail(db: &DbPool, job_id: &str, error_message: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE jobs
            SET status = 'failed',
                error_message = $1,
                completed_at = now()
            WHERE id = $2 AND status IN ('claimed', 'running')
            "#,
            error_message,
            job_id
        )
        .execute(db)
        .await
        .context(format!("failed to mark job '{}' as failed", job_id))?;

        if result.rows_affected() > 0 {
            debug!(job_id, "Job marked as failed");
        }
        Ok(result.rows_affected() > 0)
    }

    /// Cancel a pending or claimed job (admin action)
    #[instrument(skip(db), level = "debug")]
    pub async fn cancel(db: &DbPool, job_id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE jobs
            SET status = 'failed',
                error_message = 'Cancelled by admin',
                completed_at = now()
            WHERE id = $1 AND status IN ('pending', 'claimed')
            "#,
            job_id
        )
        .execute(db)
        .await
        .context(format!("failed to cancel job '{}'", job_id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Retry a failed job (admin action)
    #[instrument(skip(db), level = "debug")]
    pub async fn retry(db: &DbPool, job_id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE jobs
            SET status = 'pending',
                attempts = 0,
                error_message = NULL,
                agent_id = NULL,
                claimed_at = NULL,
                last_heartbeat = NULL,
                started_at = NULL,
                completed_at = NULL
            WHERE id = $1 AND status = 'failed'
            "#,
            job_id
        )
        .execute(db)
        .await
        .context(format!("failed to retry job '{}'", job_id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Release a claimed or running job (admin action)
    #[instrument(skip(db), level = "debug")]
    pub async fn release(db: &DbPool, job_id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE jobs
            SET status = 'pending',
                agent_id = NULL,
                claimed_at = NULL,
                last_heartbeat = NULL
            WHERE id = $1 AND status IN ('claimed', 'running')
            "#,
            job_id
        )
        .execute(db)
        .await
        .context(format!("failed to release job '{}'", job_id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Reset stale jobs that have exceeded heartbeat timeout
    #[instrument(skip(db), level = "debug")]
    pub async fn reset_stale(db: &DbPool, timeout_seconds: u64) -> AppResult<u64> {
        let timeout_secs = timeout_seconds as f64;
        let result = sqlx::query!(
            r#"
            UPDATE jobs
            SET status = CASE
                WHEN attempts >= max_attempts THEN 'failed'
                ELSE 'pending'
            END,
            error_message = CASE
                WHEN attempts >= max_attempts THEN 'Heartbeat timeout after ' || attempts || ' attempts'
                ELSE NULL
            END,
            agent_id = NULL,
            claimed_at = NULL,
            last_heartbeat = NULL
            WHERE status IN ('claimed', 'running')
              AND last_heartbeat < now() - make_interval(secs => $1)
            "#,
            timeout_secs
        )
        .execute(db)
        .await
        .context("failed to reset stale jobs")?;

        Ok(result.rows_affected())
    }

    /// Check if there are any active (claimed or running) jobs
    #[instrument(skip(db), level = "trace")]
    pub async fn has_active_jobs(db: &DbPool) -> AppResult<bool> {
        let result = sqlx::query_scalar!(
            r#"SELECT COUNT(*)::int8 as "count!" FROM jobs WHERE status IN ('claimed', 'running')"#
        )
        .fetch_one(db)
        .await
        .context("failed to check for active jobs")?;

        Ok(result > 0)
    }

    /// Count jobs by status for dashboard stats
    #[instrument(skip(db), level = "debug")]
    pub async fn count_by_status(db: &DbPool) -> AppResult<JobStatusCounts> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending')::int4 as "pending!",
                COUNT(*) FILTER (WHERE status = 'claimed')::int4 as "claimed!",
                COUNT(*) FILTER (WHERE status = 'running')::int4 as "running!",
                COUNT(*) FILTER (WHERE status = 'completed')::int4 as "completed!",
                COUNT(*) FILTER (WHERE status = 'failed')::int4 as "failed!"
            FROM jobs
            "#
        )
        .fetch_one(db)
        .await
        .context("failed to count jobs by status")?;

        Ok(JobStatusCounts {
            pending: row.pending,
            claimed: row.claimed,
            running: row.running,
            completed: row.completed,
            failed: row.failed,
        })
    }

    /// Check if job exists and is active (claimed or running)
    #[instrument(skip(db), level = "debug")]
    pub async fn is_active(db: &DbPool, job_id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!(
            "SELECT id FROM jobs WHERE id = $1 AND status IN ('claimed', 'running')",
            job_id
        )
        .fetch_optional(db)
        .await
        .context(format!("failed to check if job '{}' is active", job_id))?;

        Ok(result.is_some())
    }
}

#[derive(Debug, Serialize)]
pub struct JobStatusCounts {
    pub pending: i32,
    pub claimed: i32,
    pub running: i32,
    pub completed: i32,
    pub failed: i32,
}
