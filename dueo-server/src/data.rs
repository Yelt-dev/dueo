// Módulo `data`: export/backup e import de los datos del usuario.
// Export: vuelca categorías, suscripciones y reglas de recordatorio (scopeado por
// usuario, R13.1). Import: las RECREA en la cuenta actual remapeando los ids viejos
// a los nuevos (category_id de cada sub, subscription_id de cada regla por servicio).
// La importación AÑADE (no borra lo existente) y es atómica (una transacción).

use std::collections::HashMap;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

const EXPORT_VERSION: i64 = 1;

// ---- Formas de los datos (sirven para exportar e importar) ----------------

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryData {
    id: i64,
    name: String,
    color: Option<String>,
    icon: Option<String>,
    sort_order: i64,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionData {
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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ReminderData {
    #[serde(default)]
    id: i64,
    subscription_id: Option<i64>,
    days_before: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Backup {
    version: i64,
    #[serde(default)]
    exported_at: String,
    categories: Vec<CategoryData>,
    subscriptions: Vec<SubscriptionData>,
    reminders: Vec<ReminderData>,
}

// ---- Export ---------------------------------------------------------------

pub async fn export(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Backup>, ApiError> {
    let categories: Vec<CategoryData> = sqlx::query_as(
        "SELECT id, name, color, icon, sort_order FROM categories WHERE user_id = ? ORDER BY id",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    let subscriptions: Vec<SubscriptionData> = sqlx::query_as(
        "SELECT id, name, amount_cents, currency, cycle, cycle_days, start_date, due_date,
                category_id, payment_mode, status, notes, icon, color
         FROM subscriptions WHERE user_id = ? ORDER BY id",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    let reminders: Vec<ReminderData> = sqlx::query_as(
        "SELECT id, subscription_id, days_before FROM reminder_rules WHERE user_id = ? ORDER BY id",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    let (exported_at,): (String,) = sqlx::query_as("SELECT datetime('now')")
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    Ok(Json(Backup {
        version: EXPORT_VERSION,
        exported_at,
        categories,
        subscriptions,
        reminders,
    }))
}

// ---- Import ---------------------------------------------------------------

#[derive(Serialize)]
pub struct ImportResult {
    categories: usize,
    subscriptions: usize,
    reminders: usize,
}

pub async fn import(
    State(state): State<AppState>,
    user: AuthUser,
    Json(backup): Json<Backup>,
) -> Result<Json<ImportResult>, ApiError> {
    if backup.version != EXPORT_VERSION {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("Versión de backup no soportada (esperaba {EXPORT_VERSION})"),
        ));
    }

    let mut tx = state.db.begin().await.map_err(internal)?;

    // 1) Categorías: recrea cada una y guarda viejo_id → nuevo_id.
    let mut cat_map: HashMap<i64, i64> = HashMap::new();
    for c in &backup.categories {
        let (new_id,): (i64,) = sqlx::query_as(
            "INSERT INTO categories (user_id, name, color, icon, sort_order)
             VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(user.user_id)
        .bind(&c.name)
        .bind(&c.color)
        .bind(&c.icon)
        .bind(c.sort_order)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
        cat_map.insert(c.id, new_id);
    }

    // 2) Suscripciones: remapea category_id con el mapa anterior; guarda viejo→nuevo.
    let mut sub_map: HashMap<i64, i64> = HashMap::new();
    for s in &backup.subscriptions {
        // Hold imported rows to the same rules as a live create (the whole tx
        // rolls back on the first invalid row).
        crate::validate::subscription(
            s.amount_cents,
            &s.currency,
            &s.cycle,
            s.cycle_days,
            &s.payment_mode,
            &s.status,
        )?;
        let new_cat = s.category_id.and_then(|old| cat_map.get(&old).copied());
        let (new_id,): (i64,) = sqlx::query_as(
            "INSERT INTO subscriptions
             (user_id, name, amount_cents, currency, cycle, cycle_days, start_date, due_date,
              category_id, payment_mode, status, notes, icon, color)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(user.user_id)
        .bind(&s.name)
        .bind(s.amount_cents)
        .bind(&s.currency)
        .bind(&s.cycle)
        .bind(s.cycle_days)
        .bind(&s.start_date)
        .bind(&s.due_date)
        .bind(new_cat)
        .bind(&s.payment_mode)
        .bind(&s.status)
        .bind(&s.notes)
        .bind(&s.icon)
        .bind(&s.color)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
        sub_map.insert(s.id, new_id);
    }

    // 3) Reglas: NULL = global (se queda NULL); con valor, remapea a la sub nueva.
    //    Si la sub referida no está en el backup, se omite la regla.
    let mut reminders = 0usize;
    for r in &backup.reminders {
        crate::validate::days_before(r.days_before)?;
        let new_sub = match r.subscription_id {
            None => None,
            Some(old) => match sub_map.get(&old).copied() {
                Some(n) => Some(n),
                None => continue, // regla huérfana: la saltamos
            },
        };
        // NOT EXISTS con `IS` para no duplicar: el UNIQUE no aplica cuando
        // subscription_id es NULL (en SQLite los NULL se consideran distintos),
        // así que las reglas globales repetidas se filtran aquí.
        let res = sqlx::query(
            "INSERT INTO reminder_rules (user_id, subscription_id, days_before)
             SELECT ?, ?, ?
             WHERE NOT EXISTS (
                 SELECT 1 FROM reminder_rules
                 WHERE user_id = ? AND subscription_id IS ? AND days_before = ?
             )",
        )
        .bind(user.user_id)
        .bind(new_sub)
        .bind(r.days_before)
        .bind(user.user_id)
        .bind(new_sub)
        .bind(r.days_before)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
        reminders += res.rows_affected() as usize;
    }

    tx.commit().await.map_err(internal)?;

    Ok(Json(ImportResult {
        categories: cat_map.len(),
        subscriptions: sub_map.len(),
        reminders,
    }))
}
