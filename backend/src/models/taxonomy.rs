use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use ts_rs::TS;

/// Shader style category (realistic, fantasy, cartoon, etc.)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct Category {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

/// Shader technical feature (volumetric, PBR, ray tracing, etc.)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct Feature {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

/// Scene tag (indoor, sunset, water, etc.)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct Tag {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}
