// Telegram channel: Bot API client + per-user channel config.
// The bot TOKEN is instance-wide and read from env `DUEO_TELEGRAM_BOT_TOKEN`
// (never in code/DB). The destination (group chat_id) is stored per user in
// channel_config. For groups, chat_id is a negative number (e.g. -100123…); we
// treat it as a string to avoid precision loss. If the group has Topics, an
// optional thread_id (the topic) is stored next to it.

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
// `thread_id` = topic inside a group with Topics (None → the General chat).
pub async fn send_message(
    token: &str,
    chat_id: &str,
    thread_id: Option<&str>,
    text: &str,
) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let mut body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML",
        "disable_web_page_preview": true
    });
    if let Some(thread) = thread_id {
        body["message_thread_id"] = serde_json::json!(thread);
    }
    let resp = http()
        .post(&url)
        .json(&body)
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

// ---- Destination parsing ----------------------------------------------------

// The settings field accepts three shapes so a topic can be picked without a
// second input:
//   -1001234567890                       → whole chat
//   -1001234567890/468                   → topic 468 of that chat
//   https://t.me/c/1234567890/468/476    → link to a message inside a topic
// Topic 1 is the group's "General" chat, which takes no thread id.
pub fn parse_destination(input: &str) -> (String, Option<String>) {
    let raw = input.trim();
    let link = raw
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .strip_prefix("t.me/c/");

    let (chat, thread) = match link {
        // c/<chat>/<topic>/<msg> → topic; c/<chat>/<msg> is a message in General.
        Some(rest) => {
            let parts: Vec<&str> = rest.split('/').filter(|p| !p.is_empty()).collect();
            let chat = parts.first().map(|c| format!("-100{c}"));
            let thread = if parts.len() >= 3 {
                Some(parts[1])
            } else {
                None
            };
            match chat {
                Some(c) => (c, thread.map(str::to_owned)),
                None => (raw.to_owned(), None),
            }
        }
        None => match raw.split_once('/') {
            Some((c, t)) => (c.trim().to_owned(), Some(t.trim().to_owned())),
            None => (raw.to_owned(), None),
        },
    };

    let thread =
        thread.filter(|t| !t.is_empty() && t != "1" && t.chars().all(|ch| ch.is_ascii_digit()));
    (chat, thread)
}

// Inverse of parse_destination, for showing the stored value back in the field.
fn format_destination(chat_id: Option<String>, thread_id: Option<String>) -> Option<String> {
    match (chat_id, thread_id) {
        (Some(c), Some(t)) => Some(format!("{c}/{t}")),
        (c, _) => c,
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
    let thread_id = crate::channels::dest(&state.db, user.user_id, "telegram", "thread_id")
        .await
        .map_err(internal)?;

    Ok(Json(TelegramStatus {
        bot_ready: bot_token().is_some(),
        enabled,
        chat_id: format_destination(chat_id, thread_id),
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
    let (chat_id, thread_id) = parse_destination(&req.chat_id);
    let config = serde_json::json!({ "chat_id": chat_id, "thread_id": thread_id }).to_string();
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

    let thread_id = crate::channels::dest(&state.db, user.user_id, "telegram", "thread_id")
        .await
        .map_err(internal)?;

    let text = "✅ <b>Dueo conectado</b>\n\n\
                Este canal recibirá tus recordatorios de vencimientos. 🔔";

    match send_message(&token, &chat_id, thread_id.as_deref(), text).await {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(e) => Err((StatusCode::BAD_GATEWAY, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_destination;

    fn p(s: &str) -> (String, Option<String>) {
        parse_destination(s)
    }

    #[test]
    fn plain_chat_id_has_no_topic() {
        assert_eq!(p(" -1001234567890 "), ("-1001234567890".into(), None));
    }

    #[test]
    fn chat_slash_topic() {
        assert_eq!(
            p("-1001234567890/468"),
            ("-1001234567890".into(), Some("468".into()))
        );
    }

    #[test]
    fn topic_message_link() {
        assert_eq!(
            p("https://t.me/c/3672317911/468/476"),
            ("-1003672317911".into(), Some("468".into()))
        );
    }

    #[test]
    fn general_topic_and_plain_message_link_have_no_topic() {
        assert_eq!(
            p("https://t.me/c/3672317911/1/473"),
            ("-1003672317911".into(), None)
        );
        assert_eq!(p("t.me/c/3672317911/473"), ("-1003672317911".into(), None));
    }

    #[test]
    fn garbage_topic_is_dropped() {
        assert_eq!(p("-100123/abc"), ("-100123".into(), None));
    }
}
