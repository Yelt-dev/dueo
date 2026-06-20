// Módulo `auth`: registro, login, logout, sesión y el extractor AuthUser.

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
// Las sesiones caducan a los 30 días (se rechazan en el extractor y se limpian).
const SESSION_MAX_AGE_DAYS: i64 = 30;
// Rate-limit de login: tras 5 fallos, se bloquea ese usuario 15 min.
const LOGIN_MAX_FAILS: u32 = 5;
const LOGIN_WINDOW_SECS: u64 = 900;

// ---- Configuración por entorno --------------------------------------------

fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

// ¿Permitir auto-registro aunque ya exista admin? Por defecto NO: tras el primer
// arranque solo el admin crea cuentas (/api/users). DUEO_OPEN_REGISTRATION=1 lo abre.
fn open_registration() -> bool {
    env_flag("DUEO_OPEN_REGISTRATION")
}

// Marcar la cookie como Secure (solo viaja por HTTPS). Actívalo tras un proxy TLS
// con DUEO_SECURE_COOKIE=1. En HTTP plano (LAN) debe quedar apagado o no entra.
fn secure_cookies() -> bool {
    env_flag("DUEO_SECURE_COOKIE")
}

// Construye la cookie de sesión respetando el flag Secure.
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

// ---- Política de contraseña (compartida por register / cambio / alta admin) -

pub fn validate_password(p: &str) -> Result<(), ApiError> {
    let len = p.chars().count();
    if len < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "La contraseña debe tener al menos 8 caracteres".to_string(),
        ));
    }
    // Frases largas (12+) se aceptan tal cual; las más cortas exigen letra + número.
    let has_letter = p.chars().any(|c| c.is_alphabetic());
    let has_digit = p.chars().any(|c| c.is_ascii_digit());
    if len < 12 && !(has_letter && has_digit) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Usa al menos una letra y un número (o una frase de 12+ caracteres)".to_string(),
        ));
    }
    Ok(())
}

// ---- Rate-limit de login (en memoria, por usuario) ------------------------

fn rate_limit_check(state: &AppState, key: &str) -> Result<(), ApiError> {
    let map = state.login_attempts.lock().unwrap();
    if let Some((fails, start)) = map.get(key)
        && *fails >= LOGIN_MAX_FAILS
        && start.elapsed().as_secs() < LOGIN_WINDOW_SECS
    {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Demasiados intentos. Espera unos minutos e inténtalo de nuevo.".to_string(),
        ));
    }
    Ok(())
}

fn rate_limit_fail(state: &AppState, key: &str) {
    let mut map = state.login_attempts.lock().unwrap();
    let entry = map.entry(key.to_string()).or_insert((0, Instant::now()));
    if entry.1.elapsed().as_secs() >= LOGIN_WINDOW_SECS {
        *entry = (1, Instant::now()); // ventana vieja: reinicia el conteo
    } else {
        entry.0 += 1;
    }
}

fn rate_limit_clear(state: &AppState, key: &str) {
    state.login_attempts.lock().unwrap().remove(key);
}

// ---- Limpieza periódica de sesiones caducadas -----------------------------

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

// ---- Estado de setup (público) --------------------------------------------

// ¿Instancia recién instalada (sin usuarios)? El front muestra "crear cuenta de
// administrador" en el primer arranque. Es público (no requiere sesión).
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

// ---- Registro -------------------------------------------------------------

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<Credenciales>,
) -> Result<Json<UserRes>, ApiError> {
    // El primer usuario de la instancia se vuelve admin (R14.1).
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    // Registro cerrado salvo en el primer arranque (instancia vacía) o si se
    // habilita explícitamente. Tras el admin, las cuentas las crea el admin.
    if count > 0 && !open_registration() {
        return Err((
            StatusCode::FORBIDDEN,
            "El registro está cerrado. Pide a un administrador que cree tu cuenta.".to_string(),
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
    .map_err(unique_or_internal("El usuario ya existe"))?;

    Ok(Json(user))
}

// ---- Login / Logout -------------------------------------------------------

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<Credenciales>,
) -> Result<(CookieJar, Json<UserRes>), ApiError> {
    // Clave del rate-limit: el usuario tecleado, normalizado.
    let key = req.username.trim().to_lowercase();
    rate_limit_check(&state, &key)?; // 429 si ya superó el umbral

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
    let invalid = || {
        (
            StatusCode::UNAUTHORIZED,
            "Credenciales inválidas".to_string(),
        )
    };

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
    Ok((jar.remove(Cookie::from(SESSION_COOKIE)), "Sesión cerrada"))
}

// ---- Extractor AuthUser ---------------------------------------------------

// "El usuario autenticado de esta petición". `pub` para usarlo en otros módulos.
pub struct AuthUser {
    pub user_id: i64,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let unauthorized = || (StatusCode::UNAUTHORIZED, "No autenticado".to_string());

        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get(SESSION_COOKIE).ok_or_else(unauthorized)?;

        // Solo válida si no ha caducado (la limpieza periódica borra las viejas,
        // pero el filtro aquí la invalida de inmediato aunque aún exista la fila).
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

// ---- Ruta protegida de ejemplo --------------------------------------------

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

// ---- Preferencias de notificación (zona horaria + hora de aviso) ----------

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
    // Validaciones: zona IANA conocida y hora 0-23.
    if let Some(tz) = &req.timezone
        && tz.parse::<chrono_tz::Tz>().is_err()
    {
        return Err((StatusCode::BAD_REQUEST, "Zona horaria inválida".to_string()));
    }
    if let Some(h) = req.send_hour
        && !(0..=23).contains(&h)
    {
        return Err((StatusCode::BAD_REQUEST, "Hora inválida (0-23)".to_string()));
    }
    // Solo idiomas soportados por los mensajes del backend.
    if let Some(l) = &req.lang
        && l != "es"
        && l != "en"
    {
        return Err((StatusCode::BAD_REQUEST, "Idioma no soportado".to_string()));
    }
    if let Some(c) = &req.default_currency {
        crate::validate::currency(c)?;
    }

    // COALESCE: solo cambia lo que venga.
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

// ---- Seguridad: cambiar contraseña + cerrar todas las sesiones ------------

#[derive(Deserialize)]
pub struct ChangePassword {
    current_password: String,
    new_password: String,
}

// Cambia la contraseña verificando primero la actual (Argon2). No toca sesiones:
// el usuario sigue logueado; si quiere echar a los demás, usa /logout-all.
pub async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ChangePassword>,
) -> Result<&'static str, ApiError> {
    validate_password(&req.new_password)?;

    // Verifica la actual contra el hash guardado.
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
                "La contraseña actual no es correcta".to_string(),
            )
        })?;

    // Hashea la nueva y la guarda.
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

    Ok("Contraseña actualizada")
}

// Cierra TODAS las sesiones del usuario (incluida la actual) → queda deslogueado
// en todos los dispositivos. Borramos también la cookie de esta petición.
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
        "Sesiones cerradas",
    ))
}
