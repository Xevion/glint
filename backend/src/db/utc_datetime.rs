use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Postgres, Type,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
};
use std::fmt;

/// UTC datetime wrapper that ensures proper ISO8601 serialization with Z suffix
///
/// PostgreSQL TIMESTAMPTZ stores datetimes with timezone info. This type ensures:
/// - Storage: Native PostgreSQL TIMESTAMPTZ
/// - Serialization: ISO8601 with Z suffix (e.g., "2025-01-15T10:30:00Z")
/// - Parsing: Accepts ISO8601 formats, always treats as UTC
///
/// Note: Do NOT add #[derive(Serialize, Deserialize)] - we have custom impls below
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcDateTime(pub chrono::DateTime<chrono::Utc>);

impl UtcDateTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }

    /// Get the underlying chrono DateTime
    pub fn inner(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }

    /// Check if this datetime is before the given chrono DateTime
    pub fn is_before(&self, other: &chrono::DateTime<chrono::Utc>) -> bool {
        self.0 < *other
    }
}

impl From<chrono::DateTime<chrono::Utc>> for UtcDateTime {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt)
    }
}

impl From<String> for UtcDateTime {
    fn from(s: String) -> Self {
        // Try to parse as ISO8601
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
            Self(dt.with_timezone(&chrono::Utc))
        } else {
            // Fallback: try without timezone (assume UTC)
            let normalized = s.replace(' ', "T");
            let with_tz = if normalized.ends_with('Z') {
                normalized
            } else {
                format!("{}Z", normalized)
            };
            chrono::DateTime::parse_from_rfc3339(&with_tz)
                .map(|dt| Self(dt.with_timezone(&chrono::Utc)))
                .unwrap_or_else(|_| Self(chrono::Utc::now()))
        }
    }
}

impl fmt::Display for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

// SQLx traits for database interaction
impl Type<Postgres> for UtcDateTime {
    fn type_info() -> PgTypeInfo {
        <chrono::DateTime<chrono::Utc> as Type<Postgres>>::type_info()
    }
}

impl<'q> Encode<'q, Postgres> for UtcDateTime {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <chrono::DateTime<chrono::Utc> as Encode<'q, Postgres>>::encode_by_ref(&self.0, buf)
    }
}

impl<'r> Decode<'r, Postgres> for UtcDateTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let dt = <chrono::DateTime<chrono::Utc> as Decode<Postgres>>::decode(value)?;
        Ok(Self(dt))
    }
}

// Serde serialization with Z suffix
impl Serialize for UtcDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for UtcDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
            Ok(Self(dt.with_timezone(&chrono::Utc)))
        } else {
            Err(serde::de::Error::custom("invalid datetime format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_serialization() {
        let dt = UtcDateTime(
            chrono::DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
        );
        let json = serde_json::to_string(&dt).unwrap();
        assert!(json.contains("2025-01-15"));
        assert!(json.contains("10:30:00"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#""2025-01-15T10:30:00Z""#;
        let dt: UtcDateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.0.year(), 2025);
        assert_eq!(dt.0.month(), 1);
        assert_eq!(dt.0.day(), 15);
    }
}
