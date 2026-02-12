use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use ts_rs::TS;

use crate::id::{ShaderVersionId, ShaderVersionProfileId};

/// Extraction pipeline status for a shader version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ExtractionStatus {
    Pending,
    Completed,
    Failed,
    Skipped,
}

impl fmt::Display for ExtractionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ExtractionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ExtractionStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "pending" => Ok(Self::Pending),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            _ => Err(format!("unknown extraction status: {s}").into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ExtractionStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ExtractionStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

/// A declared profile within a specific shader version
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct ShaderVersionProfile {
    pub id: ShaderVersionProfileId,
    pub shader_version_id: ShaderVersionId,
    pub name: String,
    pub label: Option<String>,
    pub description: Option<String>,
    #[ts(type = "Record<string, string>")]
    pub options: serde_json::Value,
    pub sort_order: i32,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

/// Extracted metadata sidecar for a shader version (1:1 relationship)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct ShaderVersionMetadata {
    pub shader_version_id: ShaderVersionId,
    #[ts(optional, type = "Record<string, unknown>")]
    pub pipeline_features: Option<serde_json::Value>,
    #[ts(optional, type = "Array<string>")]
    pub iris_features_required: Option<serde_json::Value>,
    #[ts(optional, type = "Array<string>")]
    pub iris_features_optional: Option<serde_json::Value>,
    #[ts(optional, type = "Array<Record<string, unknown>>")]
    pub settings_screen: Option<serde_json::Value>,
    #[ts(optional, type = "Array<string>")]
    pub file_paths: Option<serde_json::Value>,
    #[ts(optional, type = "Array<string>")]
    pub dimension_support: Option<serde_json::Value>,
    pub has_custom_textures: Option<bool>,
    #[ts(type = "string")]
    pub extracted_at: DateTime<Utc>,
}
