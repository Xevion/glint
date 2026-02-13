use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use ts_rs::TS;

/// Shared pagination defaults.
pub const DEFAULT_PAGE_SIZE: u32 = 50;
pub const MAX_PAGE_SIZE: u32 = 250;

/// Query parameters for paginated endpoints.
///
/// Deserializable from `?page=2&page_size=25` (query string) or JSON body
/// via `#[serde(flatten)]`. Use directly as `Query<PageQuery>` for endpoints
/// that only need pagination, or flatten into a larger struct for endpoints
/// that combine pagination with filters.
///
/// `DisplayFromStr` is required because `serde(flatten)` buffers all values as
/// strings in non-self-describing formats (query strings). Without it,
/// `serde_urlencoded` fails with "invalid type: string, expected u32".
#[serde_as]
#[derive(Clone, Deserialize)]
pub struct PageQuery {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub page: Option<u32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub page_size: Option<u32>,
}

impl PageQuery {
    /// Normalize raw query parameters into a resolved [`Page`].
    pub fn normalize(&self) -> Page {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        let offset = (page - 1) as u64 * page_size as u64;
        Page {
            page,
            page_size,
            offset,
        }
    }
}

impl fmt::Debug for PageQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.page, self.page_size) {
            (None, None) => f.write_str("PageQuery(defaults)"),
            (Some(p), None) => write!(f, "PageQuery(page={p})"),
            (None, Some(s)) => write!(f, "PageQuery(size={s})"),
            (Some(p), Some(s)) => write!(f, "PageQuery(page={p}, size={s})"),
        }
    }
}

/// Resolved pagination parameters ready for SQL queries.
///
/// Produced by [`PageQuery::normalize()`]. The `offset` is pre-computed
/// from `(page - 1) * page_size`.
#[derive(Clone, Copy)]
pub struct Page {
    pub page: u32,
    pub page_size: u32,
    pub offset: u64,
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.page, self.page_size)
    }
}

impl Page {
    /// SQL-ready limit as i64 (PostgreSQL BIGINT).
    pub fn limit_i64(&self) -> i64 {
        self.page_size as i64
    }

    /// SQL-ready offset as i64 (PostgreSQL BIGINT).
    pub fn offset_i64(&self) -> i64 {
        self.offset as i64
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "page {}/{} (offset {})",
            self.page, self.page_size, self.offset
        )
    }
}

/// Generic paginated response envelope.
///
/// Used by all list endpoints that support pagination. The TypeScript binding
/// is generated as a generic `Paginated<T>` so frontend code can use
/// `Paginated<CaptureListItem>`, `Paginated<ShaderListItem>`, etc.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct Paginated<T: TS> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

impl<T: TS> Paginated<T> {
    /// Build a paginated response from items, total count, and the resolved page.
    pub fn new(items: Vec<T>, total: i64, page: &Page) -> Self {
        Self {
            items,
            total,
            page: page.page,
            page_size: page.page_size,
        }
    }
}
