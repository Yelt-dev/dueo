// Telegram channel: Bot API client + per-user channel config.
// The bot TOKEN is instance-wide and read from env `DUEO_TELEGRAM_BOT_TOKEN`
// (never in code/DB). The destination (group chat_id) is stored per user in
// channel_config. For groups, chat_id is a negative number (e.g. -100123…); we
// treat it as a string to avoid precision loss.

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

// Bot token (instance-wide). None = Telegram not configured on the server.
pub fn bot_token() -> Option<String> {
    std::env::var("DUEO_TELEGRAM_BOT_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
}

// Escape the 3 characters that break Telegram's HTML parse_mode.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// One reqwest client for the whole process so the hourly fan-out reuses the
// connection pool / TLS instead of building a client per send.
fn http() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

// Send a message via the Bot API (HTML parse_mode → bold / line breaks).
// `disable_web_page_preview` keeps links from expanding into big cards.
pub async fn send_message(token: &str, chat_id: &str, text: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let resp = http()
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
            "disable_web_page_preview": true
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("telegram {status}: {body}"))
    }
}

// ---- Channel config (endpoints) -------------------------------------------

// Telegram channel state for the UI.
#[derive(Serialize)]
pub struct TelegramStatus {
    bot_ready: bool, // is there a token on the server?
    enabled: bool,   // did the user enable the channel?
    chat_id: Option<String>,
}

pub async fn status(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<TelegramStatus>, ApiError> {
    let (enabled, chat_id) =
        crate::channels::enabled_dest(&state.db, user.user_id, "telegram", "chat_id")
            .await
            .map_err(internal)?;

    Ok(Json(TelegramStatus {
        bot_ready: bot_token().is_some(),
        enabled,
        chat_id,
    }))
}

#[derive(Deserialize)]
pub struct SetTelegram {
    chat_id: String,
    enabled: Option<bool>,
}

// Store/update the user's destination (upsert by (user_id, kind)).
pub async fn set_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SetTelegram>,
) -> Result<StatusCode, ApiError> {
    let config = serde_json::json!({ "chat_id": req.chat_id }).to_string();
    crate::channels::write_config(
        &state.db,
        user.user_id,
        "telegram",
        req.enabled.unwrap_or(true),
        &config,
    )
    .await
    .map_err(internal)?;

    Ok(StatusCode::NO_CONTENT)
}

// Send a test message to the configured chat ("send test" button, R15).
pub async fn test_send(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let token = bot_token().ok_or((
        StatusCode::BAD_REQUEST,
        "Server has no Telegram token".to_string(),
    ))?;

    let chat_id = crate::channels::dest(&state.db, user.user_id, "telegram", "chat_id")
        .await
        .map_err(internal)?
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Telegram chat not configured".to_string(),
        ))?;

    let text = "✅ <b>Dueo conectado</b>\n\n\
                Este canal recibirá tus recordatorios de vencimientos. 🔔";

    match send_message(&token, &chat_id, text).await {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(e) => Err((StatusCode::BAD_GATEWAY, e)),
    }
}
