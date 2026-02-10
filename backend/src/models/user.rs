use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

/// Discord OAuth user
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct User {
    pub id: i32,
    pub discord_id: String,
    pub discord_username: String,
    pub discord_avatar: Option<String>,
    pub role: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// Database-backed session (with in-memory cache at runtime)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub token: String,
    pub user_id: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct UserWithSessions {
    #[serde(flatten)]
    pub user: User,
    pub sessions: Vec<SessionInfo>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SessionInfo {
    pub token_prefix: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}
