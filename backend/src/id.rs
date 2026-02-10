//! Newtype ID wrappers for entity types.
//!
//! Each ID is a transparent wrapper around `String` providing type safety:
//! you can't accidentally pass a `ShaderId` where a `SceneId` is expected.
//!
//! The `define_id!` macro generates all necessary trait implementations for
//! serialization (serde), database interaction (sqlx), TypeScript binding
//! generation (ts-rs), JSON schema (schemars), and standard Rust traits.

/// Define a newtype ID wrapping `String` with all necessary trait impls.
///
/// Generates: Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
/// Display, AsRef<str>, From<String>, sqlx Type/Encode/Decode, TS, JsonSchema.
macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            Hash,
            serde::Serialize,
            serde::Deserialize,
            ts_rs::TS,
            schemars::JsonSchema,
        )]
        #[serde(transparent)]
        #[ts(export)]
        pub struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <String as sqlx::Type<sqlx::Postgres>>::type_info()
            }
        }

        impl sqlx::Encode<'_, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.0, buf)
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                <String as sqlx::Decode<sqlx::Postgres>>::decode(value).map(Self)
            }
        }
    };
}

define_id!(ShaderId);
define_id!(ShaderVersionId);
define_id!(SceneId);
define_id!(SceneVersionId);
define_id!(WorldId);
define_id!(WorldVersionId);
define_id!(CaptureId);
define_id!(CaptureRunId);
