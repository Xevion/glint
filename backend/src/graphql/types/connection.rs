use async_graphql::SimpleObject;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Relay-style page info.
#[derive(SimpleObject, Debug, Clone)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

/// Intermediate result from a cursor-paginated repo query.
/// Repos return this; callers convert it into domain-specific Connection types.
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub total_count: i64,
}

/// Trait for types that can produce a cursor string.
pub trait CursorEncodable {
    fn encode_cursor(&self) -> String;
}

impl<T: CursorEncodable> CursorPage<T> {
    /// Convert into edges (cursor + mapped node) and page info.
    pub fn into_connection<N>(self, map: impl Fn(T) -> N) -> (Vec<(String, N)>, PageInfo, i64) {
        let edges: Vec<(String, N)> = self
            .items
            .into_iter()
            .map(|item| {
                let cursor = item.encode_cursor();
                let node = map(item);
                (cursor, node)
            })
            .collect();

        let page_info = PageInfo {
            has_next_page: self.has_next_page,
            has_previous_page: self.has_previous_page,
            start_cursor: edges.first().map(|(c, _)| c.clone()),
            end_cursor: edges.last().map(|(c, _)| c.clone()),
        };

        (edges, page_info, self.total_count)
    }
}

/// Cursor payload serialized to JSON inside the base64-encoded cursor string.
#[derive(Serialize, Deserialize)]
struct CursorPayload {
    id: String,
    ts: i64,
}

/// Encode a cursor from an ID and sort timestamp.
/// Format: base64url(json({"id":"...","ts":epoch_millis}))
pub fn encode_cursor(id: &str, timestamp_millis: i64) -> String {
    let payload = CursorPayload {
        id: id.to_string(),
        ts: timestamp_millis,
    };
    let json = serde_json::to_string(&payload).expect("cursor payload is always serializable");
    URL_SAFE_NO_PAD.encode(json.as_bytes())
}

/// Decode a cursor back into (id, timestamp_millis).
pub fn decode_cursor(cursor: &str) -> AppResult<(String, i64)> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    let payload: CursorPayload = serde_json::from_slice(&bytes)
        .map_err(|_| AppError::BadRequest("Invalid cursor format".into()))?;
    Ok((payload.id, payload.ts))
}

/// Decode a cursor and convert the timestamp to a `DateTime<Utc>`.
/// Returns `(datetime, id)` ready for keyset pagination queries.
pub fn decode_cursor_for_query(cursor: &str) -> AppResult<(DateTime<Utc>, String)> {
    let (id, ts) = decode_cursor(cursor)?;
    let dt = DateTime::from_timestamp_millis(ts)
        .ok_or_else(|| AppError::BadRequest("Invalid cursor timestamp".into()))?;
    Ok((dt, id))
}
