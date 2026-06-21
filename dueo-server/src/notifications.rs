// In-app notification panel: list what the scheduler wrote to
// notification_log and mark as read. All scoped per user (R13.1).

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
    read_at: Option<String>, // NULL = unread
}

// Lists the user's notifications, most recent first.
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

// SSE stream: pushes the user's new notifications live. The client opens an
// EventSource and receives each in-app notification as soon as the scheduler
// creates it, without reloading. We filter the broadcast by user_id (R13.1 isolation).
pub async fn stream(
    State(state): State<AppState>,
    user: AuthUser,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let uid = user.user_id;
    let rx = state.tx.subscribe();
    let s = BroadcastStream::new(rx).filter_map(move |ev| match ev {
        Ok(e) if e.user_id == uid => Some(Ok(Event::default().data(e.json))),
        _ => None, // another user's, or channel lag: skip it
    });
    Sse::new(s).keep_alive(KeepAlive::default())
}

// Marks a notification as read (idempotent: no-op if it already was).
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
        return Err((StatusCode::NOT_FOUND, "Notification not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

// Marks ALL as read (for the "mark all" button).
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
