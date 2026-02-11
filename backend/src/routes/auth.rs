use axum::{
    Router,
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;
use subtle::ConstantTimeEq;
use tracing::{debug, error, warn};

use crate::{
    auth::{self, SESSION_COOKIE_NAME},
    error::{AppError, AppResult},
    repo::{SessionRepo, UserRepo, session::SESSION_DURATION_DAYS},
    state::AppState,
};

/// Cookie name for OAuth CSRF state token
const OAUTH_STATE_COOKIE: &str = "glint_oauth_state";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/discord", get(discord_login))
        .route("/discord/callback", get(discord_callback))
        .route("/logout", post(logout))
}

#[derive(Deserialize)]
pub struct LoginQuery {
    /// URL to redirect to after successful login
    redirect: Option<String>,
}

/// Validate that a redirect URL is safe (relative path only, no open redirects).
/// Rejects absolute URLs, protocol-relative URLs, and other redirect attacks.
fn validate_redirect_url(url: &str) -> &str {
    let url = url.trim();

    // Must start with "/" and must not start with "//" (protocol-relative)
    if url.starts_with('/') && !url.starts_with("//") {
        url
    } else {
        "/"
    }
}

/// GET /api/auth/discord - Initiates Discord OAuth flow
async fn discord_login(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
    jar: CookieJar,
) -> AppResult<(CookieJar, Response)> {
    let oauth_client = state
        .oauth()
        .ok_or_else(|| AppError::ServiceUnavailable("Discord OAuth not configured".to_string()))?;

    // Validate and sanitize redirect URL (prevent open redirects)
    let redirect_url = query
        .redirect
        .as_deref()
        .map(validate_redirect_url)
        .unwrap_or("/");

    // Generate CSRF state token and store it in a short-lived HttpOnly cookie
    let csrf_token = CsrfToken::new_random();
    let state_value = format!("{}:{}", csrf_token.secret(), redirect_url);

    // Store CSRF token in cookie for validation in callback
    let csrf_cookie = Cookie::build((OAUTH_STATE_COOKIE, csrf_token.secret().clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(true)
        .max_age(time::Duration::minutes(10))
        .build();
    let jar = jar.add(csrf_cookie);

    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(|| CsrfToken::new(state_value))
        .add_scope(Scope::new("identify".to_string()))
        .url();

    debug!(url = %auth_url, "Redirecting to Discord OAuth");
    Ok((jar, Redirect::temporary(auth_url.as_str()).into_response()))
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

/// Discord user info response
#[derive(Deserialize, Debug)]
struct DiscordUser {
    id: String,
    username: String,
    avatar: Option<String>,
}

/// GET /api/auth/discord/callback - Handles Discord OAuth callback
async fn discord_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
    jar: CookieJar,
) -> AppResult<(CookieJar, Response)> {
    let oauth_client = state
        .oauth()
        .ok_or_else(|| AppError::ServiceUnavailable("Discord OAuth not configured".to_string()))?;

    // Validate CSRF state: extract token from cookie and compare with state parameter
    let stored_csrf = jar
        .get(OAUTH_STATE_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or_else(|| {
            warn!("OAuth callback missing CSRF state cookie");
            AppError::BadRequest("Invalid OAuth state: missing CSRF cookie".to_string())
        })?;

    // Parse state parameter (format: "csrf_token:redirect_url")
    let (state_csrf, redirect_url) = query
        .state
        .split_once(':')
        .ok_or_else(|| AppError::BadRequest("Invalid OAuth state format".to_string()))?;

    if state_csrf.as_bytes().ct_eq(stored_csrf.as_bytes()).into() {
        // Tokens match — continue
    } else {
        warn!("OAuth CSRF token mismatch");
        return Err(AppError::BadRequest(
            "Invalid OAuth state: CSRF token mismatch".to_string(),
        ));
    }

    // Validate redirect URL again on callback (defense in depth)
    let redirect_url = validate_redirect_url(redirect_url);

    // Clear the CSRF cookie
    let clear_csrf = Cookie::build((OAUTH_STATE_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(true)
        .max_age(time::Duration::ZERO)
        .build();
    let jar = jar.add(clear_csrf);

    // Exchange authorization code for access token
    let http_client = reqwest::Client::new();
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to exchange OAuth code");
            AppError::BadRequest("Failed to exchange authorization code".to_string())
        })?;

    let access_token = token_result.access_token().secret();

    // Fetch user info from Discord
    let discord_user = fetch_discord_user(access_token).await?;
    debug!(
        discord_id = %discord_user.id,
        username = %discord_user.username,
        "Discord user authenticated"
    );

    // Upsert user in database
    let user = UserRepo::upsert(
        state.db(),
        &discord_user.id,
        &discord_user.username,
        discord_user.avatar.as_deref(),
    )
    .await?;

    // Create session (web source since this is browser OAuth)
    let session = SessionRepo::create(state.db(), user.id, "web").await?;

    // Build session cookie
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session.token))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(SESSION_DURATION_DAYS))
        .build();

    let jar = jar.add(cookie);

    debug!(
        user_id = user.id,
        "Session created, redirecting to {}", redirect_url
    );
    Ok((jar, Redirect::temporary(redirect_url).into_response()))
}

/// POST /api/auth/logout - Invalidates the current session
async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<(CookieJar, Response)> {
    // Extract session token from cookie
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let token = cookie.value();

        // Delete session from database
        if let Err(e) = auth::delete_session(state.db(), state.session_cache(), token).await {
            warn!(error = %e, "Failed to delete session from database");
        }
    }

    // Clear the cookie by setting it to expire immediately
    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::ZERO)
        .build();

    let jar = jar.add(cookie);

    Ok((jar, Redirect::temporary("/").into_response()))
}

/// Fetch user info from Discord API
async fn fetch_discord_user(access_token: &str) -> AppResult<DiscordUser> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch Discord user info");
            AppError::Internal(e.into())
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(status = %status, body = %body, "Discord API error");
        return Err(AppError::Internal(anyhow::anyhow!(
            "Discord API returned {}: {}",
            status,
            body
        )));
    }

    response.json::<DiscordUser>().await.map_err(|e| {
        error!(error = %e, "Failed to parse Discord user response");
        AppError::Internal(e.into())
    })
}
