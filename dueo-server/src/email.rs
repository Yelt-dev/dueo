// Canal Email (SMTP): cliente de envío + configuración del canal por usuario.
// La config del SERVIDOR (host/puerto/credenciales/remitente) es de la instancia y
// se lee de env (nunca en BD). El DESTINO (email del usuario) se guarda por usuario
// en channel_config (kind='email'), igual que el chat_id de Telegram.

use axum::{Json, extract::State, http::StatusCode};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

// Configuración SMTP de la instancia. None = email no configurado en el servidor.
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub from: String, // "Dueo <noreply@tu-dominio>" o un email simple
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

// Lee la config SMTP de env. Requiere al menos host y remitente (from).
pub fn smtp_config() -> Option<SmtpConfig> {
    let host = env_nonempty("DUEO_SMTP_HOST")?;
    let from = env_nonempty("DUEO_SMTP_FROM")?;
    let port = std::env::var("DUEO_SMTP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(587); // STARTTLS por defecto
    Some(SmtpConfig {
        host,
        port,
        user: env_nonempty("DUEO_SMTP_USER"),
        pass: env_nonempty("DUEO_SMTP_PASS"),
        from,
    })
}

// Reusable SMTP transport (lettre pools connections, so one mailer per run
// reuses the connection/STARTTLS handshake across a fan-out).
pub type Mailer = AsyncSmtpTransport<Tokio1Executor>;

pub fn mailer(cfg: &SmtpConfig) -> Result<Mailer, String> {
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)
        .map_err(|e| e.to_string())?
        .port(cfg.port);
    if let (Some(u), Some(p)) = (&cfg.user, &cfg.pass) {
        builder = builder.credentials(Credentials::new(u.clone(), p.clone()));
    }
    Ok(builder.build())
}

// Send one HTML email over a prebuilt mailer. Err(text) on any failure.
pub async fn send_email(
    mailer: &Mailer,
    from: &str,
    to: &str,
    subject: &str,
    html: &str,
) -> Result<(), String> {
    let email = Message::builder()
        .from(
            from.parse()
                .map_err(|e| format!("remitente inválido: {e}"))?,
        )
        .to(to.parse().map_err(|e| format!("destino inválido: {e}"))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html.to_string())
        .map_err(|e| e.to_string())?;

    mailer.send(email).await.map_err(|e| e.to_string())?;
    Ok(())
}

// ---- Configuración del canal (endpoints) ----------------------------------

#[derive(Serialize)]
pub struct EmailStatus {
    smtp_ready: bool, // ¿hay SMTP configurado en el servidor?
    enabled: bool,    // ¿el usuario activó el canal?
    email: Option<String>,
}

pub async fn status(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<EmailStatus>, ApiError> {
    let (enabled, email) = crate::channels::enabled_dest(&state.db, user.user_id, "email", "email")
        .await
        .map_err(internal)?;

    Ok(Json(EmailStatus {
        smtp_ready: smtp_config().is_some(),
        enabled,
        email,
    }))
}

#[derive(Deserialize)]
pub struct SetEmail {
    email: String,
    enabled: Option<bool>,
}

pub async fn set_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SetEmail>,
) -> Result<StatusCode, ApiError> {
    // Validación básica del email (lettre la valida en serio al enviar).
    let dest = req.email.trim();
    if !dest.is_empty() && !dest.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Email inválido".to_string()));
    }
    let config = serde_json::json!({ "email": dest }).to_string();
    crate::channels::write_config(
        &state.db,
        user.user_id,
        "email",
        req.enabled.unwrap_or(true),
        &config,
    )
    .await
    .map_err(internal)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_send(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cfg = smtp_config().ok_or((
        StatusCode::BAD_REQUEST,
        "El servidor no tiene SMTP configurado".to_string(),
    ))?;

    let dest = crate::channels::dest(&state.db, user.user_id, "email", "email")
        .await
        .map_err(internal)?
        .ok_or((
            StatusCode::BAD_REQUEST,
            "No has configurado tu email".to_string(),
        ))?;

    let html = "<h2>✅ Dueo conectado</h2>\
                <p>Este correo recibirá tus recordatorios de vencimientos. 🔔</p>";

    let m = mailer(&cfg).map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    match send_email(&m, &cfg.from, &dest, "Dueo · correo de prueba", html).await {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(e) => Err((StatusCode::BAD_GATEWAY, e)),
    }
}
