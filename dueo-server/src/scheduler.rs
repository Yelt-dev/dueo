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

// Resumen de una corrida: nuevas in-app + envíos por canal (ok/fallidos).
#[derive(Default, Serialize)]
pub struct RunReport {
    pub inapp: u64,
    pub telegram_sent: u64,
    pub telegram_failed: u64,
    pub email_sent: u64,
    pub email_failed: u64,
}

// Una fila candidata: un servicio que HOY toca avisar por una anticipación N.
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
    lang: String, // idioma del usuario (para los mensajes)
}

// ¿inglés? (cualquier otro valor → español, el idioma por defecto).
fn is_en(lang: &str) -> bool {
    lang == "en"
}

// "2026-07-01" → "1 jul 2026" / "Jul 1, 2026" según idioma (sin crate de fechas).
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

// "faltan N días" / "N days left" (0 = hoy/today, 1 = mañana/tomorrow).
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

// Versión PLAIN concisa (panel in-app + log): una línea, con emoji de tono (R12).
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

// Versión HTML para Telegram: misma estructura, con negritas. Escapa el nombre
// (texto del usuario) para no romper el parse_mode.
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

// Asunto del correo (sin HTML). Tono según el modo de pago (R12).
fn build_email_subject(c: &Candidate) -> String {
    let due = format_date(&c.due_date, &c.lang);
    match (is_en(&c.lang), c.payment_mode == "auto") {
        (true, true) => format!("Dueo · {} is charged on {}", c.name, due),
        (true, false) => format!("Dueo · {} is due on {}", c.name, due),
        (false, true) => format!("Dueo · {} se cobra el {}", c.name, due),
        (false, false) => format!("Dueo · {} vence el {}", c.name, due),
    }
}

// Cuerpo HTML del correo (email admite HTML real: usamos <br>, no \n).
fn build_email_html(c: &Candidate) -> String {
    let name = crate::telegram::html_escape(&c.name); // mismo escape sirve para HTML
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

// Avanza una fecha un ciclo (monthly/yearly/custom). None para 'once' (no recurre).
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
        _ => None, // once → no recurre
    }
}

// Mantenimiento diario del ciclo de vida (decisión del usuario):
// - Domiciliadas (payment_mode='auto') recurrentes vencidas → AUTO-RENUEVAN: ruedan
//   al siguiente ciclo (desde el vencimiento anterior, sin perder días), siguen 'active'.
// - El resto vencidas (manuales, o 'once') → pasan a 'expired' (la renueva el usuario).
// `today` = hoy en la zona del usuario. Idempotente: si ya está al día, no hace nada.
pub async fn maintain(
    db: &sqlx::SqlitePool,
    today: &str,
    only_user: Option<i64>,
) -> Result<(), sqlx::Error> {
    let Ok(today_d) = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d") else {
        return Ok(());
    };

    // 1) Auto-renovar domiciliadas recurrentes vencidas.
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
                    start = d; // el vencimiento anterior pasa a ser el nuevo inicio
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

    // 2) Expirar el resto de vencidas (manuales o 'once'); las auto-recurrentes ya
    //    rodaron arriba, así que su due_date ya no es < today.
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

// Selecciona los candidatos de un día y los escribe en el log (canal in-app).
// `today` None → date('now'). `only_user` None → todos los usuarios (cron real).
// Devuelve cuántas notificaciones NUEVAS se crearon (las repetidas se ignoran).
pub async fn run_once(
    state: &AppState,
    today: Option<String>,
    only_user: Option<i64>,
) -> Result<RunReport, sqlx::Error> {
    let db = &state.db;
    // Reglas EFECTIVAS (R11): si el servicio tiene reglas propias se usan ESAS;
    // si no, las globales del usuario (subscription_id NULL). Solo servicios que
    // generan avisos (R9): active o expired.
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
            -- reglas propias del servicio
            SELECT a.user_id, a.id AS sub_id, a.due_date, a.name, a.amount_cents,
                   a.currency, a.payment_mode, a.lang, r.days_before
            FROM active a
            JOIN reminder_rules r
              ON r.subscription_id = a.id AND r.user_id = a.user_id
            UNION ALL
            -- globales del usuario, SOLO si el servicio no tiene reglas propias
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
        let msg = build_plain(c); // guardamos SIEMPRE la versión plain en el log

        // in-app: OR IGNORE = idempotencia. RETURNING devuelve la fila SOLO si se
        // insertó (nueva) → entonces la empujamos por SSE en vivo.
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
            // send falla solo si no hay suscriptores SSE: lo ignoramos.
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

// Tarea de fondo: corre CADA HORA y, para cada usuario, evalúa sus recordatorios
// cuando en SU zona horaria es su hora de aviso (send_hour). "Hoy" se calcula en la
// zona del usuario, no en UTC. La idempotencia hace inofensivo correr de más.
pub async fn run_loop(state: AppState) {
    // Catch-up al arrancar: si la hora de aviso de hoy YA pasó, evalúa ahora
    // (para no perder el aviso tras un reinicio); si aún no llega, espera al tick.
    run_due_users(&state, true).await;

    let mut tick = tokio::time::interval(Duration::from_secs(3600));
    tick.tick().await; // consume el tick inmediato (ya hicimos el catch-up)
    loop {
        tick.tick().await;
        run_due_users(&state, false).await;
    }
}

async fn run_due_users(state: &AppState, startup: bool) {
    use chrono::Timelike;

    let users: Vec<(i64, String, i64)> =
        match sqlx::query_as("SELECT id, timezone, send_hour FROM users")
            .fetch_all(&state.db)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                eprintln!("[scheduler] no pude leer usuarios: {e}");
                return;
            }
        };

    let now_utc = chrono::Utc::now();
    for (uid, tz_name, send_hour) in users {
        let tz: chrono_tz::Tz = tz_name.parse().unwrap_or(chrono_tz::Tz::UTC);
        let local = now_utc.with_timezone(&tz);
        let hour = local.hour() as i64;

        let today = local.format("%Y-%m-%d").to_string();

        // Mantenimiento del ciclo de vida: en CADA tick (no depende de la hora de
        // aviso), para que las domiciliadas rueden y las manuales expiren a tiempo.
        if let Err(e) = maintain(&state.db, &today, Some(uid)).await {
            eprintln!("[scheduler] maintain user {uid} error: {e}");
        }

        // En tick normal: solo a la hora exacta. En arranque: catch-up si ya pasó.
        let should = if startup {
            hour >= send_hour
        } else {
            hour == send_hour
        };
        if !should {
            continue;
        }

        match run_once(state, Some(today), Some(uid)).await {
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
    }
}

// ---- Endpoint de dev: disparar el scheduler a mano ------------------------
// POST /api/scheduler/run?date=YYYY-MM-DD  (date opcional). Scopeado al usuario
// autenticado para poder probar sin afectar a otros.

#[derive(Deserialize)]
pub struct RunQuery {
    date: Option<String>,
}

pub async fn run_now(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<RunQuery>,
) -> Result<Json<RunReport>, ApiError> {
    // Mismo orden que el bucle real: primero mantenimiento, luego recordatorios.
    let today = q
        .date
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    maintain(&state.db, &today, Some(user.user_id))
        .await
        .map_err(internal)?;
    let report = run_once(&state, q.date, Some(user.user_id))
        .await
        .map_err(internal)?;
    Ok(Json(report))
}
