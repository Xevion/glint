use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite, Type,
    sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
};
use std::fmt;

/// UTC datetime wrapper that ensures proper ISO8601 serialization with Z suffix
///
/// SQLite stores datetimes as TEXT without timezone info. This type ensures:
/// - Storage: ISO8601 format (e.g., "2025-01-15 10:30:00")
/// - Serialization: ISO8601 with Z suffix (e.g., "2025-01-15T10:30:00Z")
/// - Parsing: Accepts both formats, always treats as UTC
///
/// Note: Do NOT add #[derive(Serialize, Deserialize)] - we have custom impls below
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcDateTime(String);

impl UtcDateTime {
    pub fn now() -> Self {
        // Will be set by SQL datetime('now', 'utc')
        Self(String::new())
    }

    /// Get the underlying string value (for database queries)
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this datetime is before the given chrono DateTime
    pub fn is_before(&self, other: &chrono::DateTime<chrono::Utc>) -> bool {
        // Parse the stored string and compare
        // Format: "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DDTHH:MM:SSZ"
        let normalized = self.0.replace(' ', "T");
        let with_tz = if normalized.ends_with('Z') {
            normalized
        } else {
            format!("{}Z", normalized)
        };

        match chrono::DateTime::parse_from_rfc3339(&with_tz) {
            Ok(parsed) => parsed < *other,
            Err(_) => true, // If we can't parse, treat as expired
        }
    }
}

impl From<String> for UtcDateTime {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// SQLx traits for database interaction
impl Type<Sqlite> for UtcDateTime {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for UtcDateTime {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as Encode<'q, Sqlite>>::encode_by_ref(&self.0, args)
    }
}

impl<'r> Decode<'r, Sqlite> for UtcDateTime {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<Sqlite>>::decode(value)?;
        Ok(Self(s))
    }
}

// Serde serialization with Z suffix
impl Serialize for UtcDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // SQLite stores as "YYYY-MM-DD HH:MM:SS"
        // Convert to ISO8601 with Z: "YYYY-MM-DDTHH:MM:SSZ"
        let iso8601 = if self.0.contains('T') {
            // Already in ISO format, ensure Z suffix
            if self.0.ends_with('Z') {
                self.0.clone()
            } else {
                format!("{}Z", self.0)
            }
        } else {
            // SQLite format "YYYY-MM-DD HH:MM:SS" -> "YYYY-MM-DDTHH:MM:SSZ"
            format!("{}Z", self.0.replace(' ', "T"))
        };
        serializer.serialize_str(&iso8601)
    }
}

impl<'de> Deserialize<'de> for UtcDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_space_format() {
        let dt = UtcDateTime("2025-01-15 10:30:00".to_string());
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, r#""2025-01-15T10:30:00Z""#);
    }

    #[test]
    fn test_serialization_iso_format() {
        let dt = UtcDateTime("2025-01-15T10:30:00".to_string());
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, r#""2025-01-15T10:30:00Z""#);
    }

    #[test]
    fn test_serialization_already_has_z() {
        let dt = UtcDateTime("2025-01-15T10:30:00Z".to_string());
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, r#""2025-01-15T10:30:00Z""#);
    }
}
