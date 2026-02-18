use async_graphql::{Context, Result, SimpleObject, Subscription};
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};
use tracing::warn;

use crate::graphql::events::DomainEvent;
use crate::id::{CaptureId, SceneId, ShaderVersionId};
use crate::state::AppState;

pub struct SubscriptionRoot;

#[derive(SimpleObject, Clone, Debug)]
pub struct CaptureCompletedEvent {
    pub capture_id: CaptureId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
}

#[Subscription]
impl SubscriptionRoot {
    /// Stream of newly completed captures.
    async fn capture_completed(
        &self,
        ctx: &Context<'_>,
    ) -> Result<impl Stream<Item = CaptureCompletedEvent>> {
        let state = ctx.data::<AppState>()?;
        let rx = state.event_tx().subscribe();

        Ok(BroadcastStream::new(rx).filter_map(|event| match event {
            Ok(DomainEvent::CaptureCompleted {
                capture_id,
                shader_version_id,
                scene_id,
            }) => Some(CaptureCompletedEvent {
                capture_id,
                shader_version_id,
                scene_id,
            }),
            Err(e) => {
                warn!(error = %e, "Subscription lagged, missed events");
                None
            }
        }))
    }
}
