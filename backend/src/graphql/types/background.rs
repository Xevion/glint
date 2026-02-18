use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};

use crate::id::BackgroundId;
use crate::models::{Background, ThemeMode};

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ThemeModeEnum {
    Light,
    Dark,
    Both,
}

impl From<ThemeMode> for ThemeModeEnum {
    fn from(m: ThemeMode) -> Self {
        match m {
            ThemeMode::Light => Self::Light,
            ThemeMode::Dark => Self::Dark,
            ThemeMode::Both => Self::Both,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct BackgroundNode {
    pub id: BackgroundId,
    pub image_path: String,
    pub thumbhash: Option<String>,
    pub theme_mode: ThemeModeEnum,
    pub enabled: bool,
    pub sort_order: i32,
    pub original_filename: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size_bytes: Option<i64>,
    pub content_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Background> for BackgroundNode {
    fn from(b: Background) -> Self {
        Self {
            id: b.id,
            image_path: b.image_path,
            thumbhash: b.thumbhash,
            theme_mode: b.theme_mode.into(),
            enabled: b.enabled,
            sort_order: b.sort_order,
            original_filename: b.original_filename,
            width: b.width,
            height: b.height,
            file_size_bytes: b.file_size_bytes,
            content_type: b.content_type,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}
