use crate::id::{CaptureId, SceneId, ShaderVersionId};

/// Domain events that subscription resolvers can observe.
/// Services publish these after successful state changes.
/// The broadcast channel is fire-and-forget — if no subscribers, events are dropped.
#[derive(Clone, Debug)]
pub enum DomainEvent {
    /// A capture was completed (image uploaded and metadata saved)
    CaptureCompleted {
        capture_id: CaptureId,
        shader_version_id: ShaderVersionId,
        scene_id: SceneId,
    },
}
