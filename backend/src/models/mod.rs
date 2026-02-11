pub mod adopt;
pub mod agent;
pub mod background;
pub mod capture;
pub mod device;
pub mod featured;
pub mod scene;
pub mod shader;
pub mod taxonomy;
pub mod user;
pub mod world;

// Re-export ID types for convenience
pub use crate::id::{
    BackgroundId, CaptureId, CaptureRunId, SceneId, SceneVersionId, ShaderId, ShaderVersionId,
    WorldId, WorldVersionId,
};

// Re-export all public types so existing `use crate::models::Foo` imports continue to work.

pub use adopt::{AdoptPreviewAuthor, AdoptPreviewResponse, AdoptShaderRequest, LinkShaderRequest};
pub use background::{
    Background, BackgroundUploadResponse, ConfirmBackgroundUploadRequest, ThemeMode,
    UpdateBackgroundRequest,
};
pub use capture::{
    Capture, CaptureDetail, CaptureFreshness, CaptureRun, CaptureRunItem, CaptureRunItemStatus,
    CaptureRunItemWithContext, CaptureRunStatus, CaptureStatus, CaptureWithContext,
    PaginatedCaptures,
};
pub use featured::FeaturedPair;
pub use scene::{
    Camera, CreateSceneRequest, Position, Scene, SceneListItem, SceneVersion, SceneWithCaptures,
    SceneWithVersion, SceneWithWorld, UpdateSceneMetadataRequest, UpdateSceneRequest,
};
pub use shader::{
    CreateShaderRequest, CreateShaderVersionRequest, Shader, ShaderAdopted, ShaderAuthor,
    ShaderListItem, ShaderSearchRequest, ShaderSearchResponse, ShaderSearchResult,
    ShaderSearchSort, ShaderVersion, ShaderVersionDetail, ShaderWithCaptures, ShaderWithVersions,
    TrendingShader, UpdateShaderRequest,
};
pub use taxonomy::{Category, Feature, Tag};
pub use user::{Role, Session, SessionInfo, UpdateUserRoleRequest, User, UserWithSessions};
pub use world::{
    CompleteUploadRequest, CreateWorldRequest, CreateWorldUploadRequest,
    CreateWorldVersionUploadRequest, PendingUpload, UpdateWorldRequest, UploadResponse, World,
    WorldListItem, WorldPreviewCapture, WorldVersion, WorldWithDetails,
};
