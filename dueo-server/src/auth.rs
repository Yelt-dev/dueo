// `auth` module: register, login, logout, session, and the AuthUser extractor.

use argon2::{
    Argon2, PasswordVerifier,
    password_hash::{PasswordHash, PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::{ApiError, AppState, internal, unique_or_internal};

const SESSION_COOKIE: &str = "dueo_session";
// Sessions expire after 30 days (rejected in the extractor and cleaned up).
const SESSION_MAX_AGE_DAYS: i64 = 30;
// Login rate limit: after 5 failures, that user is locked out for 15 min.
const LOGIN_MAX_FAILS: u32 = 5;
const LOGIN_WINDOW_SECS: u64 = 900;

// ---- Environment configuration --------------------------------------------

fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

// Allow self-registration even when an admin already exists? Default NO: after the
// first boot only the admin creates accounts (/api/users). DUEO_OPEN_REGISTRATION=1 opens it.
fn open_registration() -> bool {
    env_flag("DUEO_OPEN_REGISTRATION")
}

// Mark the cookie as Secure (only sent over HTTPS). Enable it behind a TLS proxy
// with DUEO_SECURE_COOKIE=1. On plain HTTP (LAN) it must stay off or the cookie won't be set.
fn secure_cookies() -> bool {
    env_flag("DUEO_SECURE_COOKIE")
}

// Build the session cookie, honoring the Secure flag.
fn session_cookie(token: String) -> Cookie<'static> {
    let b = Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/");
    if secure_cookies() {
        b.secure(true).build()
    } else {
        b.build()
    }
}

// ---- Password policy (shared by register / change / admin create) ---------

pub fn validate_password(p: &str) -> Result<(), ApiError> {
    let len = p.chars().count();
    if len < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters".to_string(),
        ));
    }
    // Long passphrases (12+) are accepted as-is; shorter ones require a letter + a digit.
    let has_letter = p.chars().any(|c| c.is_alphabetic());
    let has_digit = p.chars().any(|c| c.is_ascii_digit());
    if len < 12 && !(has_letter && has_digit) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Use at least one letter and one number (or a passphrase of 12+ characters)"
                .to_string(),
        ));
    }
    Ok(())
}

// ---- Login rate limit (in-memory, per user) -------------------------------

fn rate_limit_check(state: &AppState, key: &str) -> Result<(), ApiError> {
    let map = state.login_attempts.lock().unwrap();
    if let Some((fails, start)) = map.get(key)
        && *fails >= LOGIN_MAX_FAILS
        && start.elapsed().as_secs() < LOGIN_WINDOW_SECS
    {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts. Wait a few minutes and try again.".to_string(),
        ));
    }
    Ok(())
}

fn rate_limit_fail(state: &AppState, key: &str) {
    let mut map = state.login_attempts.lock().unwrap();
    let entry = map.entry(key.to_string()).or_insert((0, Instant::now()));
    if entry.1.elapsed().as_secs() >= LOGIN_WINDOW_SECS {
        *entry = (1, Instant::now()); // stale window: reset the counter
    } else {
        entry.0 += 1;
    }
}

fn rate_limit_clear(state: &AppState, key: &str) {
    state.login_attempts.lock().unwrap().remove(key);
}

// ---- Periodic cleanup of expired sessions ---------------------------------

pub async fn session_cleanup_loop(state: AppState) {
    loop {
        let _ = sqlx::query("DELETE FROM sessions WHERE created_at <= datetime('now', ?)")
            .bind(format!("-{SESSION_MAX_AGE_DAYS} days"))
            .execute(&state.db)
            .await;
        tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
    }
}

#[derive(Deserialize)]
pub struct Credenciales {
    username: String,
    password: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserRes {
    id: i64,
    username: String,
    role: String,
    timezone: String,
    send_hour: i64,
    default_currency: String,
    lang: String,
}

// ---- Setup status (public) ------------------------------------------------

// Freshly installed instance (no users)? The front shows "create admin account"
// on first boot. Public (no session required).
#[derive(Serialize)]
pub struct SetupStatus {
    needs_setup: bool,
    open_registration: bool,
}

pub async fn setup_status(State(state): State<AppState>) -> Result<Json<SetupStatus>, ApiError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;
    Ok(Json(SetupStatus {
        needs_setup: count == 0,
        open_registration: open_registration(),
    }))
}

// ---- Register -------------------------------------------------------------

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<Credenciales>,
) -> Result<Json<UserRes>, ApiError> {
    // The instance's first user becomes admin (R14.1).
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    // Registration is closed except on first boot (empty instance) or when
    // explicitly enabled. After the admin exists, the admin creates accounts.
    if count > 0 && !open_registration() {
        return Err((
            StatusCode::FORBIDDEN,
            "Registration is closed. Ask an administrator to create your account.".to_string(),
        ));
    }

    crate::validate::username(&req.username)?;
    validate_password(&req.password)?;

    let role = if count == 0 { "admin" } else { "member" };

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(internal)?
        .to_string();

    let user: UserRes = sqlx::query_as(
        "INSERT INTO users (username, password_hash, role)
         VALUES (?, ?, ?)
         RETURNING id, username, role, timezone, send_hour, default_currency, lang",
    )
    .bind(req.username.trim())
    .bind(&password_hash)
    .bind(role)
    .fetch_one(&state.db)
    .await
    .map_err(unique_or_internal("User already exists"))?;

    Ok(Json(user))
}

// ---- Login / Logout -------------------------------------------------------

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<Credenciales>,
) -> Result<(CookieJar, Json<UserRes>), ApiError> {
    // Rate-limit key: the typed username, normalized.
    let key = req.username.trim().to_lowercase();
    rate_limit_check(&state, &key)?; // 429 if the threshold is already exceeded

    #[derive(sqlx::FromRow)]
    struct LoginRow {
        id: i64,
        username: String,
        role: String,
        password_hash: String,
        timezone: String,
        send_hour: i64,
        default_currency: String,
        lang: String,
    }
    let row: Option<LoginRow> = sqlx::query_as(
        "SELECT id, username, role, password_hash, timezone, send_hour, default_currency, lang
         FROM users WHERE username = ? COLLATE NOCASE",
    )
    .bind(req.username.trim())
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?;

    // Generic message on purpose: don't reveal whether the user exists.
    let invalid = || (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string());

    // Unknown user counts as a failed attempt (same generic error).
    let Some(u) = row else {
        rate_limit_fail(&state, &key);
        return Err(invalid());
    };

    let parsed = PasswordHash::new(&u.password_hash).map_err(internal)?;
    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed)
        .is_err()
    {
        rate_limit_fail(&state, &key);
        return Err(invalid());
    }

    rate_limit_clear(&state, &key); // success: reset this user's fail counter

    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();
    sqlx::query("INSERT INTO sessions (id, user_id) VALUES (?, ?)")
        .bind(&token)
        .bind(u.id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    Ok((
        jar.add(session_cookie(token)),
        Json(UserRes {
            id: u.id,
            username: u.username,
            role: u.role,
            timezone: u.timezone,
            send_hour: u.send_hour,
            default_currency: u.default_currency,
            lang: u.lang,
        }),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, &'static str), ApiError> {
    if let Some(c) = jar.get(SESSION_COOKIE) {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(c.value())
            .execute(&state.db)
            .await
            .map_err(internal)?;
    }
    Ok((jar.remove(Cookie::from(SESSION_COOKIE)), "Signed out"))
}

// ---- AuthUser extractor ---------------------------------------------------

// "The authenticated user of this request." `pub` for use in other modules.
pub struct AuthUser {
    pub user_id: i64,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let unauthorized = || (StatusCode::UNAUTHORIZED, "Not authenticated".to_string());

        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get(SESSION_COOKIE).ok_or_else(unauthorized)?;

        // Only valid if not expired (the periodic cleanup deletes old rows, but
        // this filter invalidates it immediately even if the row still exists).
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT user_id FROM sessions WHERE id = ? AND created_at > datetime('now', ?)",
        )
        .bind(token.value())
        .bind(format!("-{SESSION_MAX_AGE_DAYS} days"))
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;

        let (user_id,) = row.ok_or_else(unauthorized)?;
        Ok(AuthUser { user_id })
    }
}

// ---- Example protected route ----------------------------------------------

pub async fn me(State(state): State<AppState>, user: AuthUser) -> Result<Json<UserRes>, ApiError> {
    let u: UserRes = sqlx::query_as(
        "SELECT id, username, role, timezone, send_hour, default_currency, lang FROM users WHERE id = ?",
    )
    .bind(user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;
    Ok(Json(u))
}

// ---- Notification preferences (timezone + send hour) ----------------------

#[derive(Deserialize)]
pub struct SettingsReq {
    timezone: Option<String>,
    send_hour: Option<i64>,
    default_currency: Option<String>,
    lang: Option<String>,
}

pub async fn update_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SettingsReq>,
) -> Result<Json<UserRes>, ApiError> {
    // Validation: known IANA timezone and hour 0-23.
    if let Some(tz) = &req.timezone
        && tz.parse::<chrono_tz::Tz>().is_err()
    {
        return Err((StatusCode::BAD_REQUEST, "Invalid timezone".to_string()));
    }
    if let Some(h) = req.send_hour
        && !(0..=23).contains(&h)
    {
        return Err((StatusCode::BAD_REQUEST, "Invalid hour (0-23)".to_string()));
    }
    // Only languages supported by the backend messages.
    if let Some(l) = &req.lang
        && l != "es"
        && l != "en"
    {
        return Err((StatusCode::BAD_REQUEST, "Unsupported language".to_string()));
    }
    if let Some(c) = &req.default_currency {
        crate::validate::currency(c)?;
    }

    // COALESCE: only update the fields that were provided.
    let u: UserRes = sqlx::query_as(
        "UPDATE users SET
            timezone         = COALESCE(?, timezone),
            send_hour        = COALESCE(?, send_hour),
            default_currency = COALESCE(?, default_currency),
            lang             = COALESCE(?, lang)
         WHERE id = ?
         RETURNING id, username, role, timezone, send_hour, default_currency, lang",
    )
    .bind(req.timezone)
    .bind(req.send_hour)
    .bind(req.default_currency)
    .bind(req.lang)
    .bind(user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(u))
}

// ---- Security: change password + close all sessions -----------------------

#[derive(Deserialize)]
pub struct ChangePassword {
    current_password: String,
    new_password: String,
}

// Changes the password, verifying the current one first (Argon2). Doesn't touch
// sessions: the user stays logged in; to kick out the others, use /logout-all.
pub async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ChangePassword>,
) -> Result<&'static str, ApiError> {
    validate_password(&req.new_password)?;

    let (hash,): (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
        .bind(user.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;
    let parsed = PasswordHash::new(&hash).map_err(internal)?;
    Argon2::default()
        .verify_password(req.current_password.as_bytes(), &parsed)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Current password is incorrect".to_string(),
            )
        })?;

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
        .map_err(internal)?
        .to_string();
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(user.user_id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    Ok("Password updated")
}

// Closes ALL of the user's sessions (including the current one) → logged out on
// every device. We also clear this request's cookie.
pub async fn logout_all(
    State(state): State<AppState>,
    user: AuthUser,
    jar: CookieJar,
) -> Result<(CookieJar, &'static str), ApiError> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user.user_id)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    Ok((
        jar.remove(Cookie::from(SESSION_COOKIE)),
        "All sessions closed",
    ))
}
