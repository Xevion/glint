use async_graphql::SimpleObject;

use crate::repo::capture_health::CaptureHealthSummary;

/// Aggregate capture health summary — target matrix completion status.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureHealthNode {
    pub total_targets: i32,
    pub completed: i32,
    pub missing: i32,
    pub stale: i32,
    pub failed: i32,
}

impl From<CaptureHealthSummary> for CaptureHealthNode {
    fn from(s: CaptureHealthSummary) -> Self {
        Self {
            total_targets: s.total_targets,
            completed: s.completed,
            missing: s.missing,
            stale: s.stale,
            failed: s.failed,
        }
    }
}
