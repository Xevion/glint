use async_graphql::types::connection::{self as relay, CursorType, DisableNodesField, EmptyFields};
use async_graphql::{OutputType, SimpleObject};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Cursor payload encoded into each pagination cursor.
/// Wire format: `base64url_no_pad(json({"id":"...","sv":sort_value}))`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CursorPayload {
    pub id: String,
    pub sv: i64,
}

impl CursorPayload {
    pub fn new(id: &str, sort_value: i64) -> Self {
        Self {
            id: id.to_string(),
            sv: sort_value,
        }
    }
}

/// Opaque error returned when a cursor string cannot be decoded.
#[derive(Debug)]
pub struct CursorDecodeError;

impl std::fmt::Display for CursorDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid cursor")
    }
}

impl CursorType for CursorPayload {
    type Error = CursorDecodeError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let bytes = URL_SAFE_NO_PAD.decode(s).map_err(|_| CursorDecodeError)?;
        serde_json::from_slice(&bytes).map_err(|_| CursorDecodeError)
    }

    fn encode_cursor(&self) -> String {
        let json = serde_json::to_string(self).expect("cursor payload is always serializable");
        URL_SAFE_NO_PAD.encode(json.as_bytes())
    }
}

/// Additional fields flattened onto every connection — exposes `totalCount`.
#[derive(SimpleObject, Default, Clone, Debug)]
pub struct TotalCount {
    pub total_count: i64,
}

/// Relay connection with our standard cursor format and `totalCount`.
///
/// Nodes field is disabled — consumers use `edges { node { ... } }`.
pub type Connection<N> = relay::Connection<
    CursorPayload,
    N,
    TotalCount,
    EmptyFields,
    relay::DefaultConnectionName,
    relay::DefaultEdgeName,
    DisableNodesField,
>;

/// Intermediate result from a cursor-paginated repo query.
/// Repos return this; callers convert via [`CursorPage::into_connection`].
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub total_count: i64,
}

/// Trait for model types that can produce a pagination cursor.
pub trait CursorSource {
    fn to_cursor(&self) -> CursorPayload;
}

impl<T> CursorPage<T> {
    /// Convert into a built-in async-graphql [`Connection`].
    ///
    /// `T` must implement [`CursorSource`] (to produce the cursor) and
    /// `Into<N>` (to convert the repo model into a GraphQL node).
    pub fn into_connection<N>(self) -> Connection<N>
    where
        T: CursorSource + Into<N>,
        N: OutputType,
    {
        let mut conn = relay::Connection::with_additional_fields(
            self.has_previous_page,
            self.has_next_page,
            TotalCount {
                total_count: self.total_count,
            },
        );
        conn.edges.extend(self.items.into_iter().map(|item| {
            let cursor = item.to_cursor();
            let node = item.into();
            relay::Edge::new(cursor, node)
        }));
        conn
    }
}

/// Decode a cursor string into its raw `(sort_value, id)` pair.
pub fn decode_cursor_for_query(cursor: &str) -> AppResult<(i64, String)> {
    let payload = CursorPayload::decode_cursor(cursor)
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    Ok((payload.sv, payload.id))
}

/// Decode a cursor string into `(datetime, id)` for timestamp-based keyset pagination.
pub fn decode_cursor_as_datetime(cursor: &str) -> AppResult<(DateTime<Utc>, String)> {
    let (sv, id) = decode_cursor_for_query(cursor)?;
    let dt = DateTime::from_timestamp_millis(sv)
        .ok_or_else(|| AppError::BadRequest("Invalid cursor timestamp".into()))?;
    Ok((dt, id))
}
