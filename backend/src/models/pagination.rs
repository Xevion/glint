use serde::Serialize;
use ts_rs::TS;

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
    pub page: i32,
    pub page_size: i32,
}

/// Shared pagination defaults.
pub const DEFAULT_PAGE_SIZE: i32 = 50;
pub const MAX_PAGE_SIZE: i32 = 250;

/// Parsed and clamped pagination parameters.
pub struct PaginationParams {
    pub page: i32,
    pub page_size: i32,
    pub offset: i64,
}

/// Parse and clamp pagination parameters from query strings.
pub fn normalize_pagination(page: Option<i32>, page_size: Option<i32>) -> PaginationParams {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) as i64 * page_size as i64;
    PaginationParams {
        page,
        page_size,
        offset,
    }
}
