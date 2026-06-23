// Reminder engine. Runs hourly; for each user, when it's their local send_hour
// it checks each active/expired service's effective rules (own, else the user's
// globals) and, if today is a target day (due_date - N), queues a notification.
// "Today" is computed in the user's timezone (chrono-tz), not UTC.
//
// Idempotency: UNIQUE(notification_log) + INSERT OR IGNORE means running any
// number of times yields one row per (service, channel, target due_date,
// days_before). Date math is done in SQLite (`date(due_date, '-N days')`) to
// avoid a date crate on the query path.

use std::collections::HashMap;
use std::time::Duration;

use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use crate::{ApiError, AppState, auth::AuthUser, internal};

// Summary of a run: new in-app notifications + per-channel sends (ok/failed).
#[derive(Default, Serialize)]
pub struct RunReport {
    pub inapp: u64,
    pub telegram_sent: u64,
    pub telegram_failed: u64,
    pub email_sent: u64,
    pub email_failed: u64,
}

// A candidate row: a service due to be notified TODAY for a lead time of N days.
#[derive(sqlx::FromRow)]
struct Candidate {
    user_id: i64,
    sub_id: i64,
    due_date: String,
    name: String,
    amount_cents: i64,
    currency: String,
    payment_mode: String,
    days_before: i64,
    lang: String, // user's language (for the messages)
}

// English? (any other value → Spanish, the default language).
fn is_en(lang: &str) -> bool {
    lang == "en"
}

// "2026-07-01" → "1 jul 2026" / "Jul 1, 2026" depending on language (no date crate).
fn format_date(iso: &str, lang: &str) -> String {
    const ES: [&str; 12] = [
        "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
    ];
    const EN: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = iso.split('-').collect();
    if let [y, m, d] = parts[..]
        && let (Ok(day), Ok(mon)) = (d.parse::<u32>(), m.parse::<usize>())
        && (1..=12).contains(&mon)
    {
        return if is_en(lang) {
            format!("{} {}, {}", EN[mon - 1], day, y)
        } else {
            format!("{} {} {}", day, ES[mon - 1], y)
        };
    }
    iso.to_string()
}

// "faltan N días" / "N days left" (0 = today, 1 = tomorrow).
fn days_left(n: i64, lang: &str) -> String {
    if is_en(lang) {
        match n {
            0 => "due today".to_string(),
            1 => "due tomorrow".to_string(),
            _ => format!("{n} days left"),
        }
    } else {
        match n {
            0 => "vence hoy".to_string(),
            1 => "vence mañana".to_string(),
            _ => format!("faltan {n} días"),
        }
    }
}

// Format money straight from integer cents (never via f64), honoring the
// cents invariant. Assumes 2 minor units (the app's currencies).
fn amount_str(c: &Candidate) -> String {
    let cents = c.amount_cents;
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{} {}{}.{:02}", c.currency, sign, abs / 100, abs % 100)
}

// Concise PLAIN version (in-app panel + log): one line, with a tone emoji (R12).
fn build_plain(c: &Candidate) -> String {
    let due = format_date(&c.due_date, &c.lang);
    let amount = amount_str(c);
    let left = days_left(c.days_before, &c.lang);
    match (is_en(&c.lang), c.payment_mode == "auto") {
        (true, true) => format!(
            "💳 {} is charged on {} ({}) · {}",
            c.name, due, left, amount
        ),
        (true, false) => format!("⏰ {} is due on {} ({}) · {}", c.name, due, left, amount),
        (false, true) => format!("💳 {} se cobra el {} ({}) · {}", c.name, due, left, amount),
        (false, false) => format!("⏰ {} vence el {} ({}) · {}", c.name, due, left, amount),
    }
}

// HTML version for Telegram: same structure, with bold. Escapes the name (user
// text) so it doesn't break parse_mode.
fn build_telegram(c: &Candidate) -> String {
    let name = crate::telegram::html_escape(&c.name);
    let due = format_date(&c.due_date, &c.lang);
    let amount = amount_str(c);
    let left = days_left(c.days_before, &c.lang);
    match (is_en(&c.lang), c.payment_mode == "auto") {
        (true, true) => format!(
            "💳 <b>Automatic charge coming up</b>\n\n🧾 <b>{name}</b>\n💵 {amount}\n📅 Charged on <b>{due}</b>\n⌛ {left}\n\n<i>Make sure you have funds.</i>"
        ),
        (true, false) => format!(
            "⏰ <b>Payment reminder</b>\n\n🧾 <b>{name}</b>\n💵 {amount}\n📅 Due on <b>{due}</b>\n⌛ {left}\n\n<i>Pay on time so you don't lose the service.</i>"
        ),
        (false, true) => format!(
            "💳 <b>Cobro automático próximo</b>\n\n🧾 <b>{name}</b>\n💵 {amount}\n📅 Se cobra el <b>{due}</b>\n⌛ {left}\n\n<i>Asegúrate de tener fondos.</i>"
        ),
        (false, false) => format!(
            "⏰ <b>Recordatorio de pago</b>\n\n🧾 <b>{name}</b>\n💵 {amount}\n📅 Vence el <b>{due}</b>\n⌛ {left}\n\n<i>Págalo a tiempo para no perder el servicio.</i>"
        ),
    }
}

// Email subject (no HTML). Tone depends on the payment mode (R12).
fn build_email_subject(c: &Candidate) -> String {
    let due = format_date(&c.due_date, &c.lang);
    match (is_en(&c.lang), c.payment_mode == "auto") {
        (true, true) => format!("Dueo · {} is charged on {}", c.name, due),
        (true, false) => format!("Dueo · {} is due on {}", c.name, due),
        (false, true) => format!("Dueo · {} se cobra el {}", c.name, due),
        (false, false) => format!("Dueo · {} vence el {}", c.name, due),
    }
}

// Email HTML body (email supports real HTML: we use <br>, not \n).
fn build_email_html(c: &Candidate) -> String {
    let name = crate::telegram::html_escape(&c.name); // same escape works for HTML
    let due = format_date(&c.due_date, &c.lang);
    let amount = amount_str(c);
    let left = days_left(c.days_before, &c.lang);
    match (is_en(&c.lang), c.payment_mode == "auto") {
        (true, true) => format!(
            "<h2>💳 Automatic charge coming up</h2>\
             <p><b>{name}</b><br>{amount}<br>Charged on <b>{due}</b> ({left})</p>\
             <p><i>Make sure you have funds.</i></p>"
        ),
        (true, false) => format!(
            "<h2>⏰ Payment reminder</h2>\
             <p><b>{name}</b><br>{amount}<br>Due on <b>{due}</b> ({left})</p>\
             <p><i>Pay on time so you don't lose the service.</i></p>"
        ),
        (false, true) => format!(
            "<h2>💳 Cobro automático próximo</h2>\
             <p><b>{name}</b><br>{amount}<br>Se cobra el <b>{due}</b> ({left})</p>\
             <p><i>Asegúrate de tener fondos.</i></p>"
        ),
        (false, false) => format!(
            "<h2>⏰ Recordatorio de pago</h2>\
             <p><b>{name}</b><br>{amount}<br>Vence el <b>{due}</b> ({left})</p>\
             <p><i>Págalo a tiempo para no perder el servicio.</i></p>"
        ),
    }
}

// Advances a date by one cycle (monthly/yearly/custom). None for 'once' (no recurrence).
fn add_cycle(
    due: chrono::NaiveDate,
    cycle: &str,
    cycle_days: Option<i64>,
) -> Option<chrono::NaiveDate> {
    use chrono::{Duration, Months};
    match cycle {
        "monthly" => due.checked_add_months(Months::new(1)),
        "yearly" => due.checked_add_months(Months::new(12)),
        "custom" => due.checked_add_signed(Duration::days(cycle_days.unwrap_or(30).max(1))),
        _ => None, // once → no recurrence
    }
}

// Daily lifecycle maintenance (the user's policy):
// - Overdue recurring auto-pay subscriptions (payment_mode='auto') → AUTO-RENEW: they
//   roll to the next cycle (from the previous due date, without losing days), stay 'active'.
// - The rest of the overdue ones (manual, or 'once') → move to 'expired' (user renews them).
// `today` = today in the user's timezone. Idempotent: if already up to date, does nothing.
pub async fn maintain(
    db: &sqlx::SqlitePool,
    today: &str,
    only_user: Option<i64>,
) -> Result<(), sqlx::Error> {
    let Ok(today_d) = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d") else {
        return Ok(());
    };

    // 1) Auto-renew overdue recurring auto-pay subscriptions.
    let overdue: Vec<(i64, String, String, Option<i64>)> = sqlx::query_as(
        "SELECT id, due_date, cycle, cycle_days
         FROM subscriptions
         WHERE status = 'active' AND payment_mode = 'auto'
           AND cycle IN ('monthly','yearly','custom')
           AND due_date < ?1
           AND (?2 IS NULL OR user_id = ?2)",
    )
    .bind(today)
    .bind(only_user)
    .fetch_all(db)
    .await?;

    for (id, due, cycle, cycle_days) in overdue {
        let Ok(mut d) = chrono::NaiveDate::parse_from_str(&due, "%Y-%m-%d") else {
            continue;
        };
        let mut start = d;
        let mut guard = 0;
        while d < today_d && guard < 1200 {
            match add_cycle(d, &cycle, cycle_days) {
                Some(nd) => {
                    start = d; // the previous due date becomes the new start
                    d = nd;
                }
                None => break,
            }
            guard += 1;
        }
        if d != start {
            sqlx::query("UPDATE subscriptions SET start_date = ?, due_date = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(start.format("%Y-%m-%d").to_string())
                .bind(d.format("%Y-%m-%d").to_string())
                .bind(id)
                .execute(db)
                .await?;
        }
    }

    // 2) Expire the rest of the overdue ones (manual or 'once'); the auto-recurring
    //    ones already rolled above, so their due_date is no longer < today.
    sqlx::query(
        "UPDATE subscriptions SET status = 'expired', updated_at = datetime('now')
         WHERE status = 'active' AND due_date < ?1
           AND NOT (payment_mode = 'auto' AND cycle IN ('monthly','yearly','custom'))
           AND (?2 IS NULL OR user_id = ?2)",
    )
    .bind(today)
    .bind(only_user)
    .execute(db)
    .await?;

    Ok(())
}

// Selects a day's candidates and writes them to the log (in-app channel).
// `today` None → date('now'). `only_user` None → all users (the real cron).
// Returns how many NEW notifications were created (duplicates are ignored).
pub async fn run_once(
    state: &AppState,
    today: Option<String>,
    only_user: Option<i64>,
) -> Result<RunReport, sqlx::Error> {
    let db = &state.db;
    // EFFECTIVE rules (R11): if the service has its own rules, use THOSE; otherwise
    // the user's globals (subscription_id NULL). Only services that generate
    // reminders (R9): active or expired.
    let candidates: Vec<Candidate> = sqlx::query_as(
        "WITH active AS (
            SELECT s.id, s.user_id, s.due_date, s.name, s.amount_cents, s.currency,
                   s.payment_mode, u.lang
            FROM subscriptions s
            JOIN users u ON u.id = s.user_id
            WHERE s.status IN ('active','expired')
              AND (?2 IS NULL OR s.user_id = ?2)
         ),
         eff AS (
            -- the service's own rules
            SELECT a.user_id, a.id AS sub_id, a.due_date, a.name, a.amount_cents,
                   a.currency, a.payment_mode, a.lang, r.days_before
            FROM active a
            JOIN reminder_rules r
              ON r.subscription_id = a.id AND r.user_id = a.user_id
            UNION ALL
            -- the user's globals, ONLY if the service has no rules of its own
            SELECT a.user_id, a.id, a.due_date, a.name, a.amount_cents,
                   a.currency, a.payment_mode, a.lang, g.days_before
            FROM active a
            JOIN reminder_rules g
              ON g.subscription_id IS NULL AND g.user_id = a.user_id
            WHERE NOT EXISTS (
                SELECT 1 FROM reminder_rules r2 WHERE r2.subscription_id = a.id
            )
         )
         SELECT user_id, sub_id, due_date, name, amount_cents, currency,
                payment_mode, days_before, lang
         FROM eff
         WHERE date(due_date, '-' || days_before || ' days') = COALESCE(?1, date('now'))",
    )
    .bind(today)
    .bind(only_user)
    .fetch_all(db)
    .await?;

    // Per-channel destinations (user_id → chat_id / email), loaded once per run
    // for the enabled channels only; the email mailer is built once and reused.
    let token = crate::telegram::bot_token();
    let tg = if token.is_some() {
        crate::channels::load_targets(db, "telegram", "chat_id").await?
    } else {
        HashMap::new()
    };

    let smtp = crate::email::smtp_config();
    let mailer = smtp.as_ref().and_then(|cfg| crate::email::mailer(cfg).ok());
    let em = if mailer.is_some() {
        crate::channels::load_targets(db, "email", "email").await?
    } else {
        HashMap::new()
    };

    let mut report = RunReport::default();
    for c in &candidates {
        let msg = build_plain(c); // we ALWAYS store the plain version in the log

        // in-app: OR IGNORE = idempotency. RETURNING returns the row ONLY if it was
        // inserted (new) → then we push it over SSE live.
        let inserted: Option<(i64, String)> = sqlx::query_as(
            "INSERT OR IGNORE INTO notification_log
                (user_id, subscription_id, channel, target_due_date, days_before,
                 message, sent_at)
             VALUES (?, ?, 'inapp', ?, ?, ?, datetime('now'))
             RETURNING id, created_at",
        )
        .bind(c.user_id)
        .bind(c.sub_id)
        .bind(&c.due_date)
        .bind(c.days_before)
        .bind(&msg)
        .fetch_optional(db)
        .await?;

        if let Some((id, created_at)) = inserted {
            report.inapp += 1;
            let payload = serde_json::json!({
                "id": id,
                "subscription_id": c.sub_id,
                "channel": "inapp",
                "target_due_date": c.due_date,
                "days_before": c.days_before,
                "message": msg,
                "created_at": created_at,
                "read_at": serde_json::Value::Null
            })
            .to_string();
            // send fails only when there are no SSE subscribers: ignore it.
            let _ = state.tx.send(crate::NotifEvent {
                user_id: c.user_id,
                json: payload,
            });
        }

        // Telegram: only if the user enabled it and the server has a token. The
        // rich HTML is sent; the plain `msg` is what gets logged.
        if let (Some(token), Some(chat_id)) = (token.as_ref(), tg.get(&c.user_id)) {
            let html = build_telegram(c);
            let send = crate::telegram::send_message(token, chat_id, &html);
            match crate::channels::deliver(db, &log_key(c, "telegram"), &msg, send).await {
                Ok(true) => report.telegram_sent += 1,
                Ok(false) => {}
                Err(_) => report.telegram_failed += 1,
            }
        }

        // Email: only if the user enabled it and the server has SMTP.
        if let (Some(cfg), Some(m), Some(addr)) =
            (smtp.as_ref(), mailer.as_ref(), em.get(&c.user_id))
        {
            let subject = build_email_subject(c);
            let html = build_email_html(c);
            let send = crate::email::send_email(m, &cfg.from, addr, &subject, &html);
            match crate::channels::deliver(db, &log_key(c, "email"), &msg, send).await {
                Ok(true) => report.email_sent += 1,
                Ok(false) => {}
                Err(_) => report.email_failed += 1,
            }
        }
    }
    Ok(report)
}

fn log_key<'a>(c: &'a Candidate, channel: &'a str) -> crate::channels::LogKey<'a> {
    crate::channels::LogKey {
        channel,
        user_id: c.user_id,
        sub_id: c.sub_id,
        due_date: &c.due_date,
        days_before: c.days_before,
    }
}

// Background task: runs EVERY HOUR and, for each user, evaluates their reminders
// when it's their send hour (send_hour) in THEIR timezone. "Today" is computed in
// the user's timezone, not UTC. Idempotency makes extra runs harmless.
pub async fn run_loop(state: AppState) {
    // Per-user date of the last reminder run, in memory (no persistence needed:
    // an empty map after a restart just re-evaluates today, which is idempotent).
    // It's what makes the send-hour gate survive skipped ticks: we fire once a
    // day at-or-after send_hour, not only on an exact `hour == send_hour` tick.
    let mut last_run: HashMap<i64, String> = HashMap::new();

    // Evaluate immediately on startup (catches up if today's send hour already
    // passed), then once an hour. The map keeps the extra ticks from duplicating.
    run_due_users(&state, &mut last_run).await;

    let mut tick = tokio::time::interval(Duration::from_secs(3600));
    tick.tick().await; // consume the immediate tick (we already did the catch-up)
    loop {
        tick.tick().await;
        run_due_users(&state, &mut last_run).await;
    }
}

async fn run_due_users(state: &AppState, last_run: &mut HashMap<i64, String>) {
    use chrono::Timelike;

    let users: Vec<(i64, String, i64)> =
        match sqlx::query_as("SELECT id, timezone, send_hour FROM users")
            .fetch_all(&state.db)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                eprintln!("[scheduler] failed to read users: {e}");
                return;
            }
        };

    let now_utc = chrono::Utc::now();
    for (uid, tz_name, send_hour) in users {
        let tz: chrono_tz::Tz = tz_name.parse().unwrap_or(chrono_tz::Tz::UTC);
        let local = now_utc.with_timezone(&tz);
        let hour = local.hour() as i64;

        let today = local.format("%Y-%m-%d").to_string();

        // Reminder gate: fire once per local day, at or after the user's send
        // hour. `hour >= send_hour` (not `==`) plus the last-run guard means a
        // skipped or delayed tick (suspension, throttling, clock skew) still
        // sends today's reminder instead of losing it until tomorrow.
        let already_ran = last_run.get(&uid).map(String::as_str) == Some(today.as_str());
        if hour >= send_hour && !already_ran {
            // Reminders run BEFORE maintenance so a same-day notice (e.g.
            // days_before=0) is evaluated against the current due_date, never
            // lost to a roll/expire of that very date.
            match run_once(state, Some(today.clone()), Some(uid)).await {
                Ok(r)
                    if r.inapp > 0
                        || r.telegram_sent > 0
                        || r.telegram_failed > 0
                        || r.email_sent > 0
                        || r.email_failed > 0 =>
                {
                    println!(
                        "[scheduler] user {uid}: in-app {} · telegram {}/{} · email {}/{}",
                        r.inapp, r.telegram_sent, r.telegram_failed, r.email_sent, r.email_failed
                    )
                }
                Ok(_) => {}
                Err(e) => eprintln!("[scheduler] user {uid} error: {e}"),
            }
            last_run.insert(uid, today.clone());
        }

        // Lifecycle maintenance runs on EVERY tick (independent of the send
        // hour) so auto-pay subscriptions roll and manual ones expire on time;
        // it comes AFTER the reminder gate (see above).
        if let Err(e) = maintain(&state.db, &today, Some(uid)).await {
            eprintln!("[scheduler] maintain user {uid} error: {e}");
        }
    }
}

// ---- Dev endpoint: trigger the scheduler manually -------------------------
// POST /api/scheduler/run?date=YYYY-MM-DD  (date optional). Scoped to the
// authenticated user so you can test without affecting others.

#[derive(Deserialize)]
pub struct RunQuery {
    date: Option<String>,
}

pub async fn run_now(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<RunQuery>,
) -> Result<Json<RunReport>, ApiError> {
    // Same order as the real loop: reminders FIRST, then maintenance, so a
    // same-day notice isn't lost to a roll/expire of that due date.
    let today = q
        .date
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let report = run_once(&state, q.date, Some(user.user_id))
        .await
        .map_err(internal)?;
    maintain(&state.db, &today, Some(user.user_id))
        .await
        .map_err(internal)?;
    Ok(Json(report))
}
