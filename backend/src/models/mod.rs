pub mod adopt;
pub mod agent;
pub mod background;
pub mod capture;
pub mod device;
pub mod extraction;
pub mod featured;
pub mod pagination;
pub mod scene;
pub mod shader;
pub mod stats;
pub mod storage;
pub mod taxonomy;
pub mod user;
pub mod work;

// Re-export ID types for convenience
pub use crate::id::{
    BackgroundId, CaptureId, CaptureRunId, PendingSceneUploadId, SceneId, ScenePresetId,
    SceneVersionId, ShaderId, ShaderVersionId, ShaderVersionProfileId,
};

// Re-export all public types so existing `use crate::models::Foo` imports continue to work.

pub use adopt::{AdoptPreviewAuthor, AdoptPreviewResponse, AdoptShaderRequest, LinkShaderRequest};
pub use background::{
    Background, BackgroundUploadResponse, ConfirmBackgroundUploadRequest, ThemeMode,
    UpdateBackgroundRequest,
};
pub use capture::{
    Capture, CaptureDetail, CaptureFreshness, CaptureListItem, CaptureRun, CaptureRunItem,
    CaptureRunItemStatus, CaptureRunItemWithContext, CaptureRunStatus, CaptureStatus,
    CaptureWithContext,
};
pub use extraction::{ExtractionStatus, ShaderVersionMetadata, ShaderVersionProfile};
pub use featured::FeaturedPair;
pub use pagination::{Page, PageBody, PageQuery, Paginated};
pub use scene::{
    Camera, CompleteUploadRequest, CompleteUploadResponse, CreatePresetRequest, CreateSceneRequest,
    InitiateNewSceneUploadRequest, InitiateVersionUploadRequest, PendingSceneUpload, Position,
    ReorderPresetsRequest, Scene, SceneListAdmin, SceneListItem, ScenePreset, SceneVersion,
    SceneWithCaptures, SceneWithVersion, UpdatePresetRequest, UpdateSceneMetadataRequest,
    UpdateSceneRequest, UploadInitiatedResponse,
};
pub use shader::{
    CreateShaderRequest, CreateShaderVersionRequest, ExtractionSummary, Shader, ShaderAdopted,
    ShaderAuthor, ShaderListItem, ShaderSearchRequest, ShaderSearchResponse, ShaderSearchResult,
    ShaderSearchSort, ShaderVersion, ShaderVersionDetail, ShaderWithCaptures, ShaderWithVersions,
    TrendingShader, UpdateShaderRequest,
};
pub use storage::{
    AuditObject, AuditReference, AuditSummary, CleanupKeyResult, CleanupKeyStatus,
    StorageAuditResult, StorageBucket, StorageCleanupRequest, StorageCleanupResult, StorageStats,
};
pub use taxonomy::{Category, Feature, Tag};
pub use user::{Role, Session, SessionInfo, UpdateUserRoleRequest, User, UserWithSessions};
pub use work::WorkItem;
