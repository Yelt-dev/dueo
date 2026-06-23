mod auth;
mod categories;
mod channels;
mod data;
mod email;
mod notifications;
mod reminders;
mod scheduler;
mod subscriptions;
mod telegram;
mod update;
mod users;
mod validate;

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rust_embed::RustEmbed;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// A new in-app notification to push live over SSE. `json` is already the
// serialized notification, ready for the client.
#[derive(Clone)]
pub struct NotifEvent {
    pub user_id: i64,
    pub json: String,
}

// Failed-login counter per user, in memory (no persistence needed: cleared on
// restart). Value is (failures, window start).
pub type LoginAttempts = Arc<Mutex<HashMap<String, (u32, Instant)>>>;

// State shared by every handler. `pub` so the modules can use it.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub tx: tokio::sync::broadcast::Sender<NotifEvent>,
    pub login_attempts: LoginAttempts,
    // Cached result of the GitHub update check (None until first fetch).
    pub update_cache: Arc<update::UpdateCache>,
}

// API error type and helper, shared across all modules.
pub type ApiError = (StatusCode, String);

// Log the real error server-side; never leak its Display (SQL fragments, paths)
// into the HTTP response body.
pub fn internal<E: std::fmt::Display>(e: E) -> ApiError {
    eprintln!("[error] {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal error".to_string(),
    )
}

// Map only a UNIQUE-constraint violation to 409 `msg`; any other DB error is a 500.
pub fn unique_or_internal(msg: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| match e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            (StatusCode::CONFLICT, msg.to_string())
        }
        other => internal(other),
    }
}

// Minimal `.env` reader (no crate): KEY=VALUE per line. Called at startup,
// before any thread spawns, so `set_var` is safe.
fn load_dotenv() {
    let Ok(content) = std::fs::read_to_string(".env") else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().trim_matches('"').trim_matches('\'');
            unsafe { std::env::set_var(k.trim(), v) };
        }
    }
}

#[tokio::main]
async fn main() {
    load_dotenv(); // loads the Telegram token, etc. (if a `.env` exists)

    // foreign_keys(true): SQLite does NOT enforce FKs by default. We enable it so
    // ON DELETE CASCADE works (deleting a user takes their data with it) and so
    // does ON DELETE SET NULL. It's per-connection, hence in the pool options.
    let opts = SqliteConnectOptions::from_str("sqlite:dueo.db?mode=rwc")
        .unwrap()
        .foreign_keys(true);
    let db = SqlitePool::connect_with(opts).await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();

    // Broadcast channel to push new notifications over SSE.
    let (tx, _rx) = tokio::sync::broadcast::channel::<NotifEvent>(256);
    let state = AppState {
        db,
        tx,
        login_attempts: Arc::new(Mutex::new(HashMap::new())),
        update_cache: Arc::new(Mutex::new(None)),
    };

    // Background reminder scheduler (hourly, per the user's timezone).
    tokio::spawn(scheduler::run_loop(state.clone()));
    // Periodic cleanup of expired sessions.
    tokio::spawn(auth::session_cleanup_loop(state.clone()));

    // Every API route lives under /api.
    let api = Router::new()
        .route("/health", get(health))
        // auth
        .route("/setup-status", get(auth::setup_status))
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/me", get(auth::me))
        .route("/settings", axum::routing::put(auth::update_settings))
        // security: change password and sign out everywhere
        .route("/password", post(auth::change_password))
        .route("/logout-all", post(auth::logout_all))
        // data: export/backup and import
        .route("/export", get(data::export))
        .route("/import", post(data::import))
        // users (admin only): list, create, delete
        .route("/users", get(users::list).post(users::create))
        .route("/users/{id}", axum::routing::delete(users::delete))
        // about: instance name + version (public)
        .route("/version", get(version))
        // about: is there a newer release? (notifies, never auto-updates)
        .route("/update", get(update::check))
        // subscriptions (CRUD)
        .route(
            "/subscriptions",
            get(subscriptions::list).post(subscriptions::create),
        )
        .route(
            "/subscriptions/{id}",
            get(subscriptions::get_one)
                .patch(subscriptions::update)
                .delete(subscriptions::delete),
        )
        // categories (CRUD)
        .route(
            "/categories",
            get(categories::list).post(categories::create),
        )
        .route(
            "/categories/{id}",
            axum::routing::patch(categories::update).delete(categories::delete),
        )
        // reminders (lead-time rules)
        .route("/reminders", get(reminders::list).post(reminders::create))
        .route("/reminders/{id}", axum::routing::delete(reminders::delete))
        // notifications (panel in-app)
        .route("/notifications", get(notifications::list))
        .route("/notifications/stream", get(notifications::stream))
        .route("/notifications/read", post(notifications::mark_all_read))
        .route("/notifications/{id}/read", post(notifications::mark_read))
        // scheduler (manual trigger for dev/test)
        .route("/scheduler/run", post(scheduler::run_now))
        // telegram channel: status, configure destination, send a test
        .route(
            "/channels/telegram",
            get(telegram::status).put(telegram::set_config),
        )
        .route("/channels/telegram/test", post(telegram::test_send))
        // email channel: status, configure destination, send a test
        .route("/channels/email", get(email::status).put(email::set_config))
        .route("/channels/email/test", post(email::test_send));

    // The (static SvelteKit) front is served from the binary itself: the API
    // lives under /api and ANY other route falls through to the static handler
    // (SPA fallback).
    let app = Router::new()
        .nest("/api", api)
        .fallback(static_handler)
        .layer(middleware::from_fn(security_headers))
        .with_state(state);

    // Configurable listen address. Local: 127.0.0.1:3000 (default). In a
    // container, listen on 0.0.0.0 to be reachable: DUEO_BIND=0.0.0.0:3000.
    let bind = std::env::var("DUEO_BIND").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    println!("Dueo listening on http://{bind}");
    axum::serve(listener, app).await.unwrap();
}

// Embedded front. In debug, rust-embed reads from disk (no recompile when the
// front changes); in release it embeds into the binary → a single executable.
// The folder is relative to the crate; the front must be built (pnpm build).
#[derive(RustEmbed)]
#[folder = "../dueo-web/build/"]
struct WebAssets;

// Serves the embedded file for the requested path; if it doesn't exist (an SPA
// route like /ajustes), returns index.html so the client router resolves it.
async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = WebAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response();
    }

    // SPA fallback: client routes with no file of their own → index.html.
    match WebAssets::get("index.html") {
        Some(index) => (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            index.data,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "Front not embedded. Run `pnpm build` in dueo-web.",
        )
            .into_response(),
    }
}

// About: binary name and version (from Cargo.toml at compile time).
async fn version() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// Conservative security headers on every response. No CSP (the SPA uses inline
// styles/scripts); session cookie is SameSite=Lax, which already blocks the
// cross-site requests that would carry CSRF on mutating routes.
async fn security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    h.insert("Referrer-Policy", HeaderValue::from_static("same-origin"));
    res
}

async fn health(State(state): State<AppState>) -> String {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap();
    format!("OK — registered users: {}", count.0)
}
