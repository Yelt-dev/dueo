// Shared plumbing for per-user notification channels (Telegram, email): the
// channel_config table (one row per user+kind) plus the scheduler's idempotent
// delivery bookkeeping in notification_log. Keeps the two channels DRY.

use std::collections::HashMap;
use std::future::Future;

use sqlx::SqlitePool;

// Pull a single non-empty string field (e.g. "chat_id", "email") out of a
// stored config_json blob.
fn field(json: &str, key: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v[key].as_str().map(str::to_owned))
        .filter(|s| !s.is_empty())
}

// Upsert a user's channel destination + enabled flag.
pub async fn write_config(
    db: &SqlitePool,
    user_id: i64,
    kind: &str,
    enabled: bool,
    config_json: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO channel_config (user_id, kind, enabled, config_json)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(user_id, kind)
         DO UPDATE SET enabled = excluded.enabled, config_json = excluded.config_json",
    )
    .bind(user_id)
    .bind(kind)
    .bind(enabled as i64)
    .bind(config_json)
    .execute(db)
    .await?;
    Ok(())
}

// (enabled, destination) for a channel's status endpoint.
pub async fn enabled_dest(
    db: &SqlitePool,
    user_id: i64,
    kind: &str,
    key: &str,
) -> Result<(bool, Option<String>), sqlx::Error> {
    let row: Option<(i64, String)> = sqlx::query_as(
        "SELECT enabled, config_json FROM channel_config WHERE user_id = ? AND kind = ?",
    )
    .bind(user_id)
    .bind(kind)
    .fetch_optional(db)
    .await?;
    Ok(match row {
        Some((en, cfg)) => (en != 0, field(&cfg, key)),
        None => (false, None),
    })
}

// Just the destination (test-send / scheduler single lookup).
pub async fn dest(
    db: &SqlitePool,
    user_id: i64,
    kind: &str,
    key: &str,
) -> Result<Option<String>, sqlx::Error> {
    Ok(enabled_dest(db, user_id, kind, key).await?.1)
}

// All enabled users → destination, loaded once per scheduler run (avoids N+1).
pub async fn load_targets(
    db: &SqlitePool,
    kind: &str,
    key: &str,
) -> Result<HashMap<i64, String>, sqlx::Error> {
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT user_id, config_json FROM channel_config WHERE kind = ? AND enabled = 1",
    )
    .bind(kind)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(uid, cfg)| field(&cfg, key).map(|d| (uid, d)))
        .collect())
}

// Identifies one notification_log row: the channel + the (sub, due, days_before)
// that uniquely keys idempotency, plus the owning user.
pub struct LogKey<'a> {
    pub channel: &'a str,
    pub user_id: i64,
    pub sub_id: i64,
    pub due_date: &'a str,
    pub days_before: i64,
}

// Idempotent delivery for one LogKey: skip if already sent, run `send`, then
// UPSERT sent_at on success or leave a pending row (sent_at NULL) to retry on
// failure. `message` is the plain text stored in the log (channels may send a
// richer body via `send`).
// Ok(true) = newly sent, Ok(false) = already sent, Err(()) = send failed.
pub async fn deliver<F>(
    db: &SqlitePool,
    key: &LogKey<'_>,
    message: &str,
    send: F,
) -> Result<bool, ()>
where
    F: Future<Output = Result<(), String>>,
{
    let existing: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT sent_at FROM notification_log
         WHERE subscription_id = ? AND channel = ? AND target_due_date = ? AND days_before = ?",
    )
    .bind(key.sub_id)
    .bind(key.channel)
    .bind(key.due_date)
    .bind(key.days_before)
    .fetch_optional(db)
    .await
    .map_err(|_| ())?;
    if let Some((Some(_),)) = existing {
        return Ok(false); // already delivered
    }

    match send.await {
        Ok(()) => {
            sqlx::query(
                "INSERT INTO notification_log
                    (user_id, subscription_id, channel, target_due_date, days_before, message, sent_at)
                 VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
                 ON CONFLICT(subscription_id, channel, target_due_date, days_before)
                 DO UPDATE SET sent_at = excluded.sent_at",
            )
            .bind(key.user_id)
            .bind(key.sub_id)
            .bind(key.channel)
            .bind(key.due_date)
            .bind(key.days_before)
            .bind(message)
            .execute(db)
            .await
            .map_err(|_| ())?;
            Ok(true)
        }
        Err(e) => {
            eprintln!(
                "[scheduler] {} failed (sub {}): {e}",
                key.channel, key.sub_id
            );
            // Pending row (sent_at NULL) so the next run retries.
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO notification_log
                    (user_id, subscription_id, channel, target_due_date, days_before, message)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(key.user_id)
            .bind(key.sub_id)
            .bind(key.channel)
            .bind(key.due_date)
            .bind(key.days_before)
            .bind(message)
            .execute(db)
            .await;
            Err(())
        }
    }
}
