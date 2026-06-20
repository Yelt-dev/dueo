// CRUD de reglas de anticipación (reminder_rules), scopeado por usuario.
// Una regla = "avisar N días antes". subscription_id NULL = global del usuario;
// con valor = solo ese servicio (sobrescribe a las globales, R11).
// No hay "editar": una anticipación se añade o se quita (es un valor atómico).

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
    subscription_id: Option<i64>, // NULL = global del usuario
    days_before: i64,
}

#[derive(Deserialize)]
pub struct CreateReminder {
    subscription_id: Option<i64>,
    days_before: i64,
}

// Lista todas las reglas del usuario (globales y por servicio).
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
        return Err((StatusCode::BAD_REQUEST, "days_before inválido".to_string()));
    }
    // Si la regla por servicio apunta a una sub ajena, la cortamos (R13.1).
    if let Some(sub_id) = req.subscription_id {
        let owns: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM subscriptions WHERE id = ? AND user_id = ?")
                .bind(sub_id)
                .bind(user.user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(internal)?;
        if owns.is_none() {
            return Err((
                StatusCode::NOT_FOUND,
                "Suscripción no encontrada".to_string(),
            ));
        }
    }

    // OR IGNORE para no fallar si la regla ya existe (UNIQUE). Luego la recuperamos.
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

    // `IS` para casar correctamente cuando subscription_id es NULL.
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
        return Err((StatusCode::NOT_FOUND, "Regla no encontrada".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}
