use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};

use crate::models::{Role, SessionInfo, User};

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum RoleEnum {
    User,
    Admin,
    Agent,
}

impl From<Role> for RoleEnum {
    fn from(r: Role) -> Self {
        match r {
            Role::User => Self::User,
            Role::Admin => Self::Admin,
            Role::Agent => Self::Agent,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserNode {
    pub id: i32,
    pub discord_id: String,
    pub discord_username: String,
    pub discord_avatar: Option<String>,
    pub role: RoleEnum,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserNode {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            discord_id: u.discord_id,
            discord_username: u.discord_username,
            discord_avatar: u.discord_avatar,
            role: u.role.into(),
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct SessionInfoNode {
    pub token_prefix: String,
    pub source: String,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<SessionInfo> for SessionInfoNode {
    fn from(s: SessionInfo) -> Self {
        Self {
            token_prefix: s.token_prefix,
            source: s.source,
            is_current: s.is_current,
            created_at: s.created_at,
            expires_at: s.expires_at,
        }
    }
}
