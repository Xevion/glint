use std::fmt;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use ts_rs::TS;

use crate::id::BackgroundId;

/// Which theme(s) a background image appears in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ThemeMode {
    Light,
    Dark,
    Both,
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Both => "both",
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ThemeMode {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "both" => Ok(Self::Both),
            _ => Err(format!("unknown theme mode: {s}").into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ThemeMode {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ThemeMode {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

/// A page background image stored in R2.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct Background {
    pub id: BackgroundId,
    pub image_url: String,
    pub image_path: String,
    pub thumbhash: Option<String>,
    pub theme_mode: ThemeMode,
    pub enabled: bool,
    pub sort_order: i32,
    pub original_filename: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size_bytes: Option<i64>,
    pub content_type: Option<String>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// Request body for updating background metadata.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct UpdateBackgroundRequest {
    pub enabled: Option<bool>,
    pub theme_mode: Option<ThemeMode>,
    pub sort_order: Option<i32>,
}

/// Response for initiating a background upload.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct BackgroundUploadResponse {
    pub id: BackgroundId,
    pub presigned_url: String,
    pub image_url: String,
    pub image_path: String,
}

/// Request body for confirming a background upload.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ConfirmBackgroundUploadRequest {
    pub thumbhash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size_bytes: Option<i64>,
    pub content_type: Option<String>,
}
