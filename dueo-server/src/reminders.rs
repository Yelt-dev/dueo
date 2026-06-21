// CRUD for lead-time rules (reminder_rules), scoped per user.
// A rule = "notify N days before". subscription_id NULL = user-global;
// with a value = that service only (overrides the globals, R11).
// No "edit": a lead time is added or removed (it's an atomic value).

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

#[derive(Serialize, sqlx::FromRow)]
pub struct Reminder {
    id: i64,
    subscription_id: Option<i64>, // NULL = user-global
    days_before: i64,
}

#[derive(Deserialize)]
pub struct CreateReminder {
    subscription_id: Option<i64>,
    days_before: i64,
}

// Lists all of the user's rules (global and per-service).
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Reminder>>, ApiError> {
    let rows: Vec<Reminder> = sqlx::query_as(
        "SELECT id, subscription_id, days_before
         FROM reminder_rules
         WHERE user_id = ?
         ORDER BY subscription_id IS NOT NULL, subscription_id, days_before",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(rows))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateReminder>,
) -> Result<(StatusCode, Json<Reminder>), ApiError> {
    if req.days_before < 0 {
        return Err((StatusCode::BAD_REQUEST, "invalid days_before".to_string()));
    }
    // If a per-service rule points at someone else's sub, reject it (R13.1).
    if let Some(sub_id) = req.subscription_id {
        let owns: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM subscriptions WHERE id = ? AND user_id = ?")
                .bind(sub_id)
                .bind(user.user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(internal)?;
        if owns.is_none() {
            return Err((StatusCode::NOT_FOUND, "Subscription not found".to_string()));
        }
    }

    // OR IGNORE so we don't fail if the rule already exists (UNIQUE). Then read it back.
    sqlx::query(
        "INSERT OR IGNORE INTO reminder_rules (user_id, subscription_id, days_before)
         VALUES (?, ?, ?)",
    )
    .bind(user.user_id)
    .bind(req.subscription_id)
    .bind(req.days_before)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    // `IS` to match correctly when subscription_id is NULL.
    let rule: Reminder = sqlx::query_as(
        "SELECT id, subscription_id, days_before
         FROM reminder_rules
         WHERE user_id = ? AND subscription_id IS ? AND days_before = ?",
    )
    .bind(user.user_id)
    .bind(req.subscription_id)
    .bind(req.days_before)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query("DELETE FROM reminder_rules WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.user_id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Rule not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}
