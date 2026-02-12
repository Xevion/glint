use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use ts_rs::TS;

/// User roles with compile-time safety.
///
/// Stored as lowercase text in PostgreSQL (`user`, `admin`, `agent`).
/// Using an enum prevents typo-based auth bypasses that string comparisons allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Role {
    User,
    Admin,
    Agent,
}

impl Role {
    /// Whether this role grants admin privileges.
    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Whether this role grants agent-level access (agents and admins).
    pub fn has_agent_access(self) -> bool {
        matches!(self, Role::Agent | Role::Admin)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Admin => write!(f, "admin"),
            Role::Agent => write!(f, "agent"),
        }
    }
}

/// Discord OAuth user
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct User {
    pub id: i32,
    pub discord_id: String,
    pub discord_username: String,
    pub discord_avatar: Option<String>,
    pub role: Role,
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
    /// How the session was created: "web" (browser OAuth) or "device" (mod)
    pub source: String,
    /// Whether this is the session making the current request
    pub is_current: bool,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct UpdateUserRoleRequest {
    pub role: Role,
}
