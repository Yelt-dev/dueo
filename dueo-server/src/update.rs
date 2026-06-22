// Update check: compares this build's version against the latest GitHub release
// and tells the UI whether a newer one exists. It only NOTIFIES — a container
// can't replace itself from the inside; the front shows how to update.
//
// Privacy: this is a plain GET to GitHub's public API (no data leaves the
// instance), but it's still an outbound call, so it's opt-out via the env
// `DUEO_UPDATE_CHECK` (default ON) and documented in the README.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser};

// Public GitHub repo we release from.
const RELEASES_LATEST: &str = "https://api.github.com/repos/Yelt-dev/dueo/releases/latest";
// How long a successful check stays fresh. GitHub's unauthenticated limit is
// 60 req/h per IP; with a 24h cache we never get close.
const TTL: Duration = Duration::from_secs(24 * 60 * 60);

// What the front gets. `checked:false` means we didn't (couldn't) ask GitHub —
// either opt-out is on or the network failed and there's no cached answer.
#[derive(Clone, Serialize)]
pub struct UpdateInfo {
    current: String,
    latest: Option<String>,
    update_available: bool,
    url: Option<String>,
    published_at: Option<String>,
    checked: bool,
}

// In-memory cache shared via AppState: (when we fetched, what we got).
pub type UpdateCache = Mutex<Option<(Instant, UpdateInfo)>>;

// Is the check enabled? Default ON; `DUEO_UPDATE_CHECK=0` (or false/no/off)
// turns it off.
fn enabled() -> bool {
    match std::env::var("DUEO_UPDATE_CHECK") {
        Ok(v) => !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"),
        Err(_) => true,
    }
}

// Process-wide reqwest client (reused connection pool / TLS).
fn http() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

// Parse "v1.2.3" / "1.2.3-rc1" into (major, minor, patch); ignores anything
// after the first '-' (pre-release) and any trailing junk. Missing parts = 0.
fn parse_semver(s: &str) -> (u64, u64, u64) {
    let core = s.trim().trim_start_matches('v');
    let core = core.split('-').next().unwrap_or(core);
    let mut it = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
    )
}

// Only the fields we need from the GitHub release JSON.
#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    published_at: Option<String>,
}

// Ask GitHub for the latest release. User-Agent is mandatory (GitHub rejects
// requests without one). Returns Err on any network / parse / non-2xx failure.
async fn fetch_latest(current: &str) -> Result<UpdateInfo, String> {
    let resp = http()
        .get(RELEASES_LATEST)
        .header(reqwest::header::USER_AGENT, "dueo-update-check")
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("github {}", resp.status()));
    }

    let rel: GhRelease = resp.json().await.map_err(|e| e.to_string())?;
    let update_available = parse_semver(&rel.tag_name) > parse_semver(current);

    Ok(UpdateInfo {
        current: current.to_string(),
        latest: Some(rel.tag_name),
        update_available,
        url: Some(rel.html_url),
        published_at: rel.published_at,
        checked: true,
    })
}

// GET /api/update — version status for "About". Never errors out to the client
// over the network: on failure it falls back to a cached answer, or to a
// `checked:false` payload, so the settings page always renders.
pub async fn check(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<UpdateInfo>, ApiError> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    // Opt-out: report only the current version, don't reach out.
    if !enabled() {
        return Ok(Json(UpdateInfo {
            current,
            latest: None,
            update_available: false,
            url: None,
            published_at: None,
            checked: false,
        }));
    }

    // Fresh cache hit? (Lock scoped so it's never held across the await below.)
    {
        let cache = state.update_cache.lock().unwrap();
        if let Some((at, info)) = cache.as_ref()
            && at.elapsed() < TTL
        {
            return Ok(Json(info.clone()));
        }
    }

    match fetch_latest(&current).await {
        Ok(info) => {
            *state.update_cache.lock().unwrap() = Some((Instant::now(), info.clone()));
            Ok(Json(info))
        }
        // Network/GitHub failure: serve a stale cached answer if we have one,
        // otherwise degrade gracefully to checked:false (no error to the UI).
        Err(e) => {
            eprintln!("[update] check failed: {e}");
            if let Some((_, info)) = state.update_cache.lock().unwrap().as_ref() {
                return Ok(Json(info.clone()));
            }
            Ok(Json(UpdateInfo {
                current,
                latest: None,
                update_available: false,
                url: None,
                published_at: None,
                checked: false,
            }))
        }
    }
}
