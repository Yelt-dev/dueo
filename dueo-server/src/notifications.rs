// Panel de notificaciones in-app: listar lo que el scheduler dejó en
// notification_log y marcar como leído. Todo scopeado por usuario (R13.1).

use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use serde::Serialize;
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};

use crate::{ApiError, AppState, auth::AuthUser, internal};

#[derive(Serialize, sqlx::FromRow)]
pub struct Notification {
    id: i64,
    subscription_id: i64,
    channel: String,
    target_due_date: String,
    days_before: i64,
    message: String,
    created_at: String,
    read_at: Option<String>, // NULL = no leída
}

// Lista las notificaciones del usuario, más recientes primero.
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Notification>>, ApiError> {
    let rows: Vec<Notification> = sqlx::query_as(
        "SELECT id, subscription_id, channel, target_due_date, days_before,
                message, created_at, read_at
         FROM notification_log
         WHERE user_id = ?
         ORDER BY created_at DESC, id DESC",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(rows))
}

// Stream SSE: empuja en vivo las notificaciones nuevas del usuario. El cliente
// abre un EventSource y recibe cada notificación in-app en cuanto el scheduler la
// crea, sin recargar. Filtramos el broadcast por user_id (aislamiento R13.1).
pub async fn stream(
    State(state): State<AppState>,
    user: AuthUser,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let uid = user.user_id;
    let rx = state.tx.subscribe();
    let s = BroadcastStream::new(rx).filter_map(move |ev| match ev {
        Ok(e) if e.user_id == uid => Some(Ok(Event::default().data(e.json))),
        _ => None, // de otro usuario, o lag del canal: lo saltamos
    });
    Sse::new(s).keep_alive(KeepAlive::default())
}

// Marca una notificación como leída (idempotente: si ya estaba, no pasa nada).
pub async fn mark_read(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query(
        "UPDATE notification_log
         SET read_at = COALESCE(read_at, datetime('now'))
         WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user.user_id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    if res.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Notificación no encontrada".to_string(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

// Marca TODAS como leídas (para el botón "marcar todo").
pub async fn mark_all_read(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<StatusCode, ApiError> {
    sqlx::query(
        "UPDATE notification_log
         SET read_at = datetime('now')
         WHERE user_id = ? AND read_at IS NULL",
    )
    .bind(user.user_id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    Ok(StatusCode::NO_CONTENT)
}
