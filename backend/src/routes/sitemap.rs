use std::fmt::Write;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::{
    error::{AppError, AppResult},
    repo::SitemapRepo,
    state::AppState,
};

/// TTL for the cached sitemap XML (15 minutes).
const CACHE_TTL: Duration = Duration::from_secs(15 * 60);

struct CachedSitemap {
    xml: String,
    generated_at: Instant,
}

static SITEMAP_CACHE: LazyLock<RwLock<Option<CachedSitemap>>> = LazyLock::new(|| RwLock::new(None));

/// GET /api/sitemap.xml
#[instrument(skip(state))]
pub async fn sitemap(State(state): State<AppState>) -> Result<Response, AppError> {
    // Check cache first
    {
        let cache = SITEMAP_CACHE.read().await;
        if let Some(ref cached) = *cache
            && cached.generated_at.elapsed() < CACHE_TTL
        {
            debug!("Serving cached sitemap");
            return Ok(xml_response(&cached.xml));
        }
    }

    // Cache miss or expired — regenerate
    let xml = generate_sitemap(state.db()).await?;

    // Store in cache
    {
        let mut cache = SITEMAP_CACHE.write().await;
        *cache = Some(CachedSitemap {
            xml: xml.clone(),
            generated_at: Instant::now(),
        });
    }

    debug!("Generated fresh sitemap");
    Ok(xml_response(&xml))
}

async fn generate_sitemap(db: &crate::db::DbPool) -> AppResult<String> {
    let (shaders, scenes, authors) = tokio::try_join!(
        SitemapRepo::list_shader_entries(db),
        SitemapRepo::list_scene_entries(db),
        SitemapRepo::list_author_entries(db),
    )?;

    let mut xml = String::with_capacity(4096);
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Static pages
    write_url(&mut xml, "/", None);
    write_url(&mut xml, "/compare", None);

    for entry in &shaders {
        write_url(
            &mut xml,
            &format!("/shaders/{}", entry.slug),
            Some(entry.last_modified),
        );
    }

    for entry in &scenes {
        write_url(
            &mut xml,
            &format!("/scenes/{}", entry.slug),
            Some(entry.last_modified),
        );
    }

    // Author pages (placeholder — routes don't exist yet)
    for entry in &authors {
        write_url(
            &mut xml,
            &format!("/authors/{}", xml_escape(&entry.name)),
            Some(entry.last_modified),
        );
    }

    xml.push_str("</urlset>\n");
    Ok(xml)
}

fn write_url(xml: &mut String, path: &str, last_modified: Option<DateTime<Utc>>) {
    let escaped = xml_escape(path);
    let _ = writeln!(xml, "  <url>\n    <loc>{escaped}</loc>");
    if let Some(ts) = last_modified {
        let _ = writeln!(xml, "    <lastmod>{}</lastmod>", ts.format("%Y-%m-%d"));
    }
    let _ = writeln!(xml, "  </url>");
}

/// Escape XML special characters in sitemap values.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

fn xml_response(body: &str) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        body.to_owned(),
    )
        .into_response()
}
