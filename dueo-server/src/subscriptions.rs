// `subscriptions` module: CRUD for the core domain, scoped per user.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

// Distinguishes "key absent" from "key = null". With `default`, a missing field
// → None; a present field (value or null) goes through here and is wrapped in
// Some, so `null` arrives as Some(None) (intent: set NULL).
fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(de)?))
}

// A category can only be attached if it belongs to the same user (keeps the
// foreign-row-is-404 invariant; mirrors the owns-check in reminders.rs).
async fn category_belongs(
    db: &sqlx::SqlitePool,
    id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM categories WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(db)
            .await?;
    Ok(row.is_some())
}

// What goes OUT as JSON. FromRow = sqlx builds it from a row.
#[derive(Serialize, sqlx::FromRow)]
pub struct Subscription {
    id: i64,
    name: String,
    amount_cents: i64,
    currency: String,
    cycle: String,
    cycle_days: Option<i64>,
    start_date: String,
    due_date: String,
    category_id: Option<i64>,
    payment_mode: String,
    status: String,
    notes: Option<String>,
    icon: Option<String>,
    color: Option<String>,
}

// What comes IN on create. The Option fields get defaults in the handler.
#[derive(Deserialize)]
pub struct CreateSub {
    name: String,
    amount_cents: i64,
    currency: Option<String>,
    cycle: String,
    cycle_days: Option<i64>,
    start_date: String,
    due_date: String,
    category_id: Option<i64>,
    payment_mode: Option<String>,
    notes: Option<String>,
    icon: Option<String>,
    color: Option<String>,
}

// ---- Create ---------------------------------------------------------------

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateSub>,
) -> Result<(StatusCode, Json<Subscription>), ApiError> {
    let currency = req.currency.as_deref().unwrap_or("USD");
    let payment_mode = req.payment_mode.as_deref().unwrap_or("manual");
    crate::validate::subscription(
        req.amount_cents,
        currency,
        &req.cycle,
        req.cycle_days,
        payment_mode,
        "active",
    )?;
    if let Some(cid) = req.category_id
        && !category_belongs(&state.db, cid, user.user_id)
            .await
            .map_err(internal)?
    {
        return Err((StatusCode::NOT_FOUND, "Category not found".to_string()));
    }

    let sub: Subscription = sqlx::query_as(
        "INSERT INTO subscriptions
         (user_id, name, amount_cents, currency, cycle, cycle_days,
          start_date, due_date, category_id, payment_mode, notes, icon, color)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING id, name, amount_cents, currency, cycle, cycle_days,
                   start_date, due_date, category_id, payment_mode, status, notes,
                   icon, color",
    )
    .bind(user.user_id) // <- the owner
    .bind(&req.name)
    .bind(req.amount_cents)
    .bind(currency)
    .bind(&req.cycle)
    .bind(req.cycle_days)
    .bind(&req.start_date)
    .bind(&req.due_date)
    .bind(req.category_id)
    .bind(payment_mode)
    .bind(&req.notes)
    .bind(&req.icon)
    .bind(&req.color)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(sub)))
}

// ---- List (only the user's, soonest due first) ----------------------------

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Subscription>>, ApiError> {
    let subs: Vec<Subscription> = sqlx::query_as(
        "SELECT id, name, amount_cents, currency, cycle, cycle_days,
                start_date, due_date, category_id, payment_mode, status, notes,
                icon, color
         FROM subscriptions
         WHERE user_id = ?
         ORDER BY due_date ASC",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(subs))
}

// ---- Get one --------------------------------------------------------------

pub async fn get_one(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Subscription>, ApiError> {
    // The `AND user_id = ?` makes another user's row "not exist" (404), not 403.
    let sub: Subscription = sqlx::query_as(
        "SELECT id, name, amount_cents, currency, cycle, cycle_days,
                start_date, due_date, category_id, payment_mode, status, notes,
                icon, color
         FROM subscriptions
         WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or((StatusCode::NOT_FOUND, "Subscription not found".to_string()))?;

    Ok(Json(sub))
}

// ---- Update (partial, via COALESCE) ---------------------------------------

// All optional: only the fields that were sent get updated. COALESCE(?, col) =
// "use the new value if present; otherwise keep the current one".
#[derive(Deserialize)]
pub struct UpdateSub {
    name: Option<String>,
    amount_cents: Option<i64>,
    currency: Option<String>,
    cycle: Option<String>,
    cycle_days: Option<i64>,
    start_date: Option<String>,
    due_date: Option<String>,
    // Double Option: distinguish "key absent" (None → leave untouched) from
    // "sent as null" (Some(None) → set NULL = clear the category).
    // `default` so a PATCH that omits it (e.g. renewal) doesn't wipe it.
    #[serde(default, deserialize_with = "double_option")]
    category_id: Option<Option<i64>>,
    payment_mode: Option<String>,
    status: Option<String>,
    notes: Option<String>,
    // icon/color also double-Option: null = re-enable autodetection.
    #[serde(default, deserialize_with = "double_option")]
    icon: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    color: Option<Option<String>>,
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSub>,
) -> Result<Json<Subscription>, ApiError> {
    // Validate only the fields that were actually sent.
    if let Some(a) = req.amount_cents {
        crate::validate::amount(a)?;
    }
    if let Some(c) = &req.currency {
        crate::validate::currency(c)?;
    }
    if let Some(c) = &req.cycle {
        crate::validate::cycle(c)?;
    }
    if let Some(m) = &req.payment_mode {
        crate::validate::payment_mode(m)?;
    }
    if let Some(s) = &req.status {
        crate::validate::status(s)?;
    }
    if let Some(Some(cid)) = req.category_id
        && !category_belongs(&state.db, cid, user.user_id)
            .await
            .map_err(internal)?
    {
        return Err((StatusCode::NOT_FOUND, "Category not found".to_string()));
    }

    let sub: Option<Subscription> = sqlx::query_as(
        "UPDATE subscriptions SET
            name         = COALESCE(?, name),
            amount_cents = COALESCE(?, amount_cents),
            currency     = COALESCE(?, currency),
            cycle        = COALESCE(?, cycle),
            cycle_days   = COALESCE(?, cycle_days),
            start_date   = COALESCE(?, start_date),
            due_date     = COALESCE(?, due_date),
            -- if the first ? is true (key was present), use the second ?
            -- (which may be NULL to clear it); otherwise keep the current value.
            category_id  = CASE WHEN ? THEN ? ELSE category_id END,
            payment_mode = COALESCE(?, payment_mode),
            status       = COALESCE(?, status),
            notes        = COALESCE(?, notes),
            icon         = CASE WHEN ? THEN ? ELSE icon END,
            color        = CASE WHEN ? THEN ? ELSE color END,
            updated_at   = datetime('now')
         WHERE id = ? AND user_id = ?
         RETURNING id, name, amount_cents, currency, cycle, cycle_days,
                   start_date, due_date, category_id, payment_mode, status, notes,
                   icon, color",
    )
    .bind(req.name)
    .bind(req.amount_cents)
    .bind(req.currency)
    .bind(req.cycle)
    .bind(req.cycle_days)
    .bind(req.start_date)
    .bind(req.due_date)
    .bind(req.category_id.is_some()) // was the category_id key present?
    .bind(req.category_id.flatten()) // its value (Some(id) or None=NULL)
    .bind(req.payment_mode)
    .bind(req.status)
    .bind(req.notes)
    .bind(req.icon.is_some()) // was icon present?
    .bind(req.icon.flatten()) // its value (or NULL = autodetect)
    .bind(req.color.is_some()) // was color present?
    .bind(req.color.flatten()) // its value (or NULL)
    .bind(id)
    .bind(user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?;

    sub.map(Json)
        .ok_or((StatusCode::NOT_FOUND, "Subscription not found".to_string()))
}

// ---- Delete ---------------------------------------------------------------

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query("DELETE FROM subscriptions WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.user_id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Subscription not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT) // 204: deleted OK, no body
}
