use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

/// Shader style category (realistic, fantasy, cartoon, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Category {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

/// Shader technical feature (volumetric, PBR, ray tracing, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Feature {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

/// Scene tag (indoor, sunset, water, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Tag {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}
