pub mod adopt;
pub mod agent;
pub mod capture;
pub mod device;
pub mod featured;
pub mod scene;
pub mod shader;
pub mod taxonomy;
pub mod user;
pub mod world;

// Re-export all public types so existing `use crate::models::Foo` imports continue to work.

pub use adopt::{AdoptPreviewAuthor, AdoptPreviewResponse, AdoptShaderRequest, LinkShaderRequest};
pub use capture::{
    Capture, CaptureDetail, CaptureFreshness, CaptureRun, CaptureRunItem,
    CaptureRunItemWithContext, CaptureWithContext, PaginatedCaptures,
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
    UpdateShaderRequest,
};
pub use taxonomy::{Category, Feature, Tag};
pub use user::{Session, SessionInfo, UpdateUserRoleRequest, User, UserWithSessions};
pub use world::{
    CompleteUploadRequest, CreateWorldRequest, CreateWorldUploadRequest,
    CreateWorldVersionUploadRequest, PendingUpload, UpdateWorldRequest, UploadResponse, World,
    WorldListItem, WorldPreviewCapture, WorldVersion, WorldWithDetails,
};
