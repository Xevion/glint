use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use mini_moka::sync::Cache;
use tracing::{debug, instrument, trace};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::{Session, User};
use crate::repo::SessionRepo;

/// Cached (User, Session) pair stored in the session cache.
type CachedSession = (User, Session);

const DEFAULT_TTL_SECS: u64 = 60;
const DEFAULT_MAX_CAPACITY: u64 = 10_000;

/// In-memory session cache that sits between auth extractors and the database.
///
/// Uses mini-moka for TTL-based eviction and a DashMap reverse index to support
/// efficient per-user invalidation without clearing the entire cache.
#[derive(Clone)]
pub struct SessionCache {
    inner: Arc<SessionCacheInner>,
}

struct SessionCacheInner {
    /// token -> (User, Session)
    cache: Cache<String, CachedSession>,
    /// user_id -> set of cached tokens (reverse index for per-user invalidation)
    user_tokens: DashMap<i32, HashSet<String>>,
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionCache {
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(DEFAULT_TTL_SECS))
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(DEFAULT_MAX_CAPACITY)
            .time_to_live(ttl)
            .build();

        Self {
            inner: Arc::new(SessionCacheInner {
                cache,
                user_tokens: DashMap::new(),
            }),
        }
    }

    /// Look up a session by token, using the cache when possible.
    /// On cache miss, validates against the database and populates the cache.
    #[instrument(skip(self, db, token), level = "debug")]
    pub async fn get_or_validate(&self, db: &DbPool, token: &str) -> AppResult<(User, Session)> {
        if let Some(cached) = self.inner.cache.get(&token.to_string()) {
            trace!("Session cache hit");
            return Ok(cached);
        }

        trace!("Session cache miss, validating against database");
        let result = SessionRepo::validate(db, token).await?;

        // Populate cache and reverse index
        let token_owned = token.to_string();
        self.inner.cache.insert(token_owned.clone(), result.clone());
        self.inner
            .user_tokens
            .entry(result.0.id)
            .or_default()
            .insert(token_owned);

        Ok(result)
    }

    /// Evict a single session from the cache by token.
    pub fn invalidate_token(&self, token: &str) {
        self.inner.cache.invalidate(&token.to_string());
        // Clean up reverse index — we don't know the user_id here, so scan all entries.
        // With few users in alpha this is negligible.
        self.inner.user_tokens.retain(|_user_id, tokens| {
            tokens.remove(token);
            !tokens.is_empty()
        });
    }

    /// Evict all cached sessions for a specific user.
    pub fn invalidate_user(&self, user_id: i32) {
        if let Some((_uid, tokens)) = self.inner.user_tokens.remove(&user_id) {
            let count = tokens.len();
            for token in tokens {
                self.inner.cache.invalidate(&token);
            }
            debug!(user_id, count, "Invalidated cached sessions for user");
        }
    }

    /// Remove stale entries from the `user_tokens` reverse index.
    ///
    /// Tokens that have been evicted from the moka cache via TTL are still
    /// referenced in the DashMap. This method scans the reverse index and
    /// removes tokens that are no longer in the cache, dropping empty user
    /// entries entirely.
    pub fn cleanup_stale_tokens(&self) {
        let mut removed = 0usize;
        self.inner.user_tokens.retain(|_user_id, tokens| {
            let before = tokens.len();
            tokens.retain(|t| self.inner.cache.get(t).is_some());
            removed += before - tokens.len();
            !tokens.is_empty()
        });
        if removed > 0 {
            debug!(removed, "Cleaned up stale session cache tokens");
        }
    }

    /// Directly insert into cache and reverse index (for testing without DB).
    #[cfg(test)]
    fn seed(&self, user_id: i32, token: &str, user: User, session: Session) {
        let token_owned = token.to_string();
        self.inner
            .cache
            .insert(token_owned.clone(), (user, session));
        self.inner
            .user_tokens
            .entry(user_id)
            .or_default()
            .insert(token_owned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Role;
    use chrono::Utc;

    fn make_user(id: i32) -> User {
        User {
            id,
            discord_id: format!("{id}"),
            discord_username: format!("user{id}"),
            discord_avatar: None,
            role: Role::User,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_session(user_id: i32, token: &str) -> Session {
        Session {
            token: token.into(),
            user_id,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
        }
    }

    fn seed(cache: &SessionCache, user_id: i32, token: &str) {
        cache.seed(
            user_id,
            token,
            make_user(user_id),
            make_session(user_id, token),
        );
    }

    #[test]
    fn cache_hit_after_seed() {
        let cache = SessionCache::new();
        seed(&cache, 1, "aaaa1111bbbb2222");

        assert!(
            cache
                .inner
                .cache
                .get(&"aaaa1111bbbb2222".to_string())
                .is_some()
        );
    }

    #[test]
    fn invalidate_token_removes_from_cache_and_index() {
        let cache = SessionCache::new();
        seed(&cache, 1, "aaaa1111bbbb2222");

        cache.invalidate_token("aaaa1111bbbb2222");

        assert!(
            cache
                .inner
                .cache
                .get(&"aaaa1111bbbb2222".to_string())
                .is_none()
        );
        assert!(cache.inner.user_tokens.get(&1).is_none());
    }

    #[test]
    fn invalidate_token_leaves_other_tokens_intact() {
        let cache = SessionCache::new();
        seed(&cache, 1, "token_a");
        seed(&cache, 1, "token_b");

        cache.invalidate_token("token_a");

        assert!(cache.inner.cache.get(&"token_a".to_string()).is_none());
        assert!(cache.inner.cache.get(&"token_b".to_string()).is_some());
        let tokens = cache.inner.user_tokens.get(&1).unwrap();
        assert!(tokens.contains("token_b"));
        assert!(!tokens.contains("token_a"));
    }

    #[test]
    fn invalidate_user_removes_all_tokens_for_user() {
        let cache = SessionCache::new();
        seed(&cache, 1, "token_a");
        seed(&cache, 1, "token_b");
        seed(&cache, 2, "token_c");

        cache.invalidate_user(1);

        assert!(cache.inner.cache.get(&"token_a".to_string()).is_none());
        assert!(cache.inner.cache.get(&"token_b".to_string()).is_none());
        assert!(cache.inner.cache.get(&"token_c".to_string()).is_some());
        assert!(cache.inner.user_tokens.get(&1).is_none());
        assert!(cache.inner.user_tokens.get(&2).is_some());
    }

    #[test]
    fn invalidate_user_noop_for_unknown_user() {
        let cache = SessionCache::new();
        seed(&cache, 1, "token_a");

        cache.invalidate_user(999);

        assert!(cache.inner.cache.get(&"token_a".to_string()).is_some());
    }

    #[tokio::test]
    async fn ttl_expiration_evicts_entry() {
        let cache = SessionCache::with_ttl(Duration::from_millis(50));
        seed(&cache, 1, "expires_soon");

        assert!(cache.inner.cache.get(&"expires_soon".to_string()).is_some());
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(cache.inner.cache.get(&"expires_soon".to_string()).is_none());
    }

    #[tokio::test]
    async fn cleanup_stale_tokens_removes_expired_entries() {
        let cache = SessionCache::with_ttl(Duration::from_millis(50));
        seed(&cache, 1, "token_a");
        seed(&cache, 1, "token_b");
        seed(&cache, 2, "token_c");

        // Reverse index has entries for both users
        assert!(cache.inner.user_tokens.get(&1).is_some());
        assert!(cache.inner.user_tokens.get(&2).is_some());

        // Wait for TTL expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Tokens are gone from moka cache but reverse index is stale
        assert!(cache.inner.cache.get(&"token_a".to_string()).is_none());
        assert!(cache.inner.user_tokens.get(&1).is_some());

        // Cleanup should remove all stale entries
        cache.cleanup_stale_tokens();

        assert!(cache.inner.user_tokens.get(&1).is_none());
        assert!(cache.inner.user_tokens.get(&2).is_none());
    }
}
