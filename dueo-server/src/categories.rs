// `categories` module: CRUD for categories, scoped per user (like subscriptions).
// The table already exists in migration 0003. Deleting a category leaves the subs
// that used it with category_id = NULL (ON DELETE SET NULL); they are not deleted.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

// What goes OUT as JSON. FromRow = sqlx builds it from a row.
#[derive(Serialize, sqlx::FromRow)]
pub struct Category {
    id: i64,
    name: String,
    color: Option<String>,
    icon: Option<String>,
    sort_order: i64,
}

// What comes IN on create. The Option fields get defaults in the handler.
#[derive(Deserialize)]
pub struct CreateCat {
    name: String,
    color: Option<String>,
    icon: Option<String>,
    sort_order: Option<i64>,
}

// ---- Create ---------------------------------------------------------------

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateCat>,
) -> Result<(StatusCode, Json<Category>), ApiError> {
    let cat: Category = sqlx::query_as(
        "INSERT INTO categories (user_id, name, color, icon, sort_order)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, name, color, icon, sort_order",
    )
    .bind(user.user_id) // <- the owner
    .bind(&req.name)
    .bind(&req.color)
    .bind(&req.icon)
    .bind(req.sort_order.unwrap_or(0))
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    Ok((StatusCode::CREATED, Json(cat)))
}

// ---- List (only the user's, in their manual order) ------------------------

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Category>>, ApiError> {
    let cats: Vec<Category> = sqlx::query_as(
        "SELECT id, name, color, icon, sort_order
         FROM categories
         WHERE user_id = ?
         ORDER BY sort_order ASC, name ASC",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(cats))
}

// ---- Update (partial, via COALESCE) ---------------------------------------

#[derive(Deserialize)]
pub struct UpdateCat {
    name: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    sort_order: Option<i64>,
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateCat>,
) -> Result<Json<Category>, ApiError> {
    let cat: Option<Category> = sqlx::query_as(
        "UPDATE categories SET
            name       = COALESCE(?, name),
            color      = COALESCE(?, color),
            icon       = COALESCE(?, icon),
            sort_order = COALESCE(?, sort_order)
         WHERE id = ? AND user_id = ?
         RETURNING id, name, color, icon, sort_order",
    )
    .bind(req.name)
    .bind(req.color)
    .bind(req.icon)
    .bind(req.sort_order)
    .bind(id)
    .bind(user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?;

    cat.map(Json)
        .ok_or((StatusCode::NOT_FOUND, "Category not found".to_string()))
}

// ---- Delete ---------------------------------------------------------------
// The subs that used it are left with category_id = NULL (ON DELETE SET NULL).

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query("DELETE FROM categories WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.user_id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Category not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT) // 204: deleted OK, no body
}
