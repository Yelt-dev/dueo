// Canal Telegram: cliente de la Bot API + configuración del canal por usuario.
// El TOKEN del bot es de la instancia y se lee de la env `DUEO_TELEGRAM_BOT_TOKEN`
// (nunca en código/BD). El destino (chat_id del grupo) lo guarda cada usuario en
// channel_config. Para grupos, chat_id es un número negativo (p.ej. -100123…); lo
// tratamos como string para no perder precisión.

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

// Token del bot (de la instancia). None = Telegram no configurado en el servidor.
pub fn bot_token() -> Option<String> {
    std::env::var("DUEO_TELEGRAM_BOT_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
}

// Escapa los 3 caracteres que rompen el parse_mode HTML de Telegram.
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

// ---- Configuración del canal (endpoints) ----------------------------------

// Estado del canal Telegram para la UI.
#[derive(Serialize)]
pub struct TelegramStatus {
    bot_ready: bool, // ¿hay token en el servidor?
    enabled: bool,   // ¿el usuario activó el canal?
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

// Guarda/actualiza el destino del usuario (upsert por (user_id, kind)).
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

// Envía un mensaje de prueba al chat configurado (botón "enviar prueba", R15).
pub async fn test_send(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let token = bot_token().ok_or((
        StatusCode::BAD_REQUEST,
        "El servidor no tiene token de Telegram".to_string(),
    ))?;

    let chat_id = crate::channels::dest(&state.db, user.user_id, "telegram", "chat_id")
        .await
        .map_err(internal)?
        .ok_or((
            StatusCode::BAD_REQUEST,
            "No has configurado el chat de Telegram".to_string(),
        ))?;

    let text = "✅ <b>Dueo conectado</b>\n\n\
                Este canal recibirá tus recordatorios de vencimientos. 🔔";

    match send_message(&token, &chat_id, text).await {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(e) => Err((StatusCode::BAD_GATEWAY, e)),
    }
}
