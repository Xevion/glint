use async_graphql::Enum;

/// Controls whether hidden (disabled, uncaptured) items are included in query results.
///
/// - `Exclude` (default): Only publicly visible items. No auth required.
/// - `Include`: Visible + hidden items. Requires admin.
/// - `Only`: Only hidden items. Requires admin.
#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Visibility {
    /// Only publicly visible items (default, no auth required).
    #[default]
    Exclude,
    /// Both visible and hidden items (requires admin).
    Include,
    /// Only hidden items (requires admin).
    Only,
}

impl Visibility {
    /// Whether this visibility level requires admin privileges.
    pub fn requires_admin(self) -> bool {
        !matches!(self, Self::Exclude)
    }
}
