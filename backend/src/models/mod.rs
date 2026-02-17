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
pub mod world;

// Re-export ID types for convenience
pub use crate::id::{
    BackgroundId, CaptureId, CaptureRunId, SceneId, SceneVersionId, ShaderId, ShaderVersionId,
    ShaderVersionProfileId, WorldId, WorldVersionId,
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
pub use pagination::{Page, PageQuery, Paginated};
pub use scene::{
    Camera, CreateSceneRequest, Position, Scene, SceneListItem, SceneVersion, SceneWithCaptures,
    SceneWithVersion, SceneWithWorld, UpdateSceneMetadataRequest, UpdateSceneRequest,
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
pub use world::{
    CompleteUploadRequest, CreateWorldRequest, CreateWorldUploadRequest,
    CreateWorldVersionUploadRequest, PendingUpload, UpdateWorldRequest, UploadResponse, World,
    WorldListItem, WorldPreviewCapture, WorldVersion, WorldWithDetails,
};
