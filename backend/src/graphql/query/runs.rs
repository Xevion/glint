use async_graphql::{Context, Object, Result};

use crate::graphql::guard::AdminGuard;
use crate::graphql::types::run::{CaptureRunItemNode, CaptureRunNode};
use crate::repo::CaptureRunRepo;
use crate::state::AppState;

#[derive(Default)]
pub struct RunQuery;

#[Object]
impl RunQuery {
    /// Get a capture run by ID with all items (admin only).
    #[graphql(guard = "AdminGuard")]
    async fn admin_capture_run(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Capture run ID")] id: String,
    ) -> Result<Option<CaptureRunNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();

        let run = match CaptureRunRepo::get_by_id(db, &id).await {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        let items = CaptureRunRepo::list_items_with_context(db, &id).await?;

        let item_nodes: Vec<CaptureRunItemNode> = items
            .into_iter()
            .map(|item| CaptureRunItemNode {
                id: item.id,
                run_id: item.run_id,
                shader_version_id: item.shader_version_id,
                scene_id: item.scene_id,
                profile_id: item.profile_id,
                profile_name: item.profile_name,
                profile_display_name: item.profile_display_name,
                preset_id: item.preset_id,
                status: item.status.into(),
                capture_id: item.capture_id,
                error_message: item.error_message,
                error_log: item.error_log,
                duration_ms: item.duration_ms,
                started_at: item.started_at,
                completed_at: item.completed_at,
                shader_name: item.shader_name,
                shader_slug: item.shader_slug,
                shader_version: item.shader_version,
                scene_name: item.scene_name,
                image_path: item.image_path,
                thumbhash: item.thumbhash,
            })
            .collect();

        Ok(Some(CaptureRunNode {
            id: run.id,
            agent_id: run.agent_id,
            started_at: run.started_at,
            completed_at: run.completed_at,
            status: run.status.into(),
            total_items: run.total_items,
            completed_items: run.completed_items,
            failed_items: run.failed_items,
            skipped_items: run.skipped_items,
            items: item_nodes,
        }))
    }
}
