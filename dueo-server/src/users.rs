// `users` module: account management (admin only). In a self-hosted instance
// the admin creates/removes everyone else. Deleting a user drags ALL their data
// along via FK ON DELETE CASCADE (requires foreign_keys = ON on the pool, see main.rs).

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

// Guard: requires the requesting user to be an admin (R14). 403 if not.
pub async fn require_admin(state: &AppState, user_id: i64) -> Result<(), ApiError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;
    match row {
        Some((role,)) if role == "admin" => Ok(()),
        _ => Err((
            StatusCode::FORBIDDEN,
            "Administrator privileges required".to_string(),
        )),
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserRow {
    id: i64,
    username: String,
    role: String,
    created_at: String,
}

// ---- List (admin) ---------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<UserRow>>, ApiError> {
    require_admin(&state, user.user_id).await?;

    let users: Vec<UserRow> =
        sqlx::query_as("SELECT id, username, role, created_at FROM users ORDER BY created_at ASC")
            .fetch_all(&state.db)
            .await
            .map_err(internal)?;

    Ok(Json(users))
}

// ---- Create (admin) -------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    password: String,
    role: Option<String>, // 'admin' | 'member' (default member)
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateUser>,
) -> Result<(StatusCode, Json<UserRow>), ApiError> {
    require_admin(&state, user.user_id).await?;

    crate::validate::username(&req.username)?;
    crate::auth::validate_password(&req.password)?;
    let role = match req.role.as_deref() {
        Some("admin") => "admin",
        _ => "member",
    };

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(internal)?
        .to_string();

    let created: UserRow = sqlx::query_as(
        "INSERT INTO users (username, password_hash, role)
         VALUES (?, ?, ?)
         RETURNING id, username, role, created_at",
    )
    .bind(req.username.trim())
    .bind(&password_hash)
    .bind(role)
    .fetch_one(&state.db)
    .await
    .map_err(crate::unique_or_internal("User already exists"))?;

    Ok((StatusCode::CREATED, Json(created)))
}

// ---- Delete (admin) -------------------------------------------------------
// Safeguards: can't delete yourself, nor leave the instance with no admin.

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state, user.user_id).await?;

    if id == user.user_id {
        return Err((
            StatusCode::BAD_REQUEST,
            "You can't delete your own account".to_string(),
        ));
    }

    // Atomic last-admin guard: delete only if the target isn't the sole admin,
    // closing the COUNT-then-DELETE race that could leave zero admins. FK cascade
    // removes the user's sessions, categories, subs, rules, etc.
    let res = sqlx::query(
        "DELETE FROM users
         WHERE id = ?1
           AND (role <> 'admin' OR (SELECT COUNT(*) FROM users WHERE role = 'admin') > 1)",
    )
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    if res.rows_affected() == 0 {
        // Nothing deleted: disambiguate not-found from last-admin for the message.
        let target: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?;
        return match target {
            Some(_) => Err((
                StatusCode::BAD_REQUEST,
                "You can't delete the only administrator".to_string(),
            )),
            None => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
        };
    }

    Ok(StatusCode::NO_CONTENT)
}
