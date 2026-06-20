// Shared input validators, used by both the live API handlers and the import path
// so a restored backup is held to the same rules as a normal create.

use axum::http::StatusCode;

use crate::ApiError;

const CYCLES: [&str; 4] = ["once", "monthly", "yearly", "custom"];
const STATUSES: [&str; 5] = ["active", "paused", "expired", "cancelled", "archived"];
const PAYMENT_MODES: [&str; 2] = ["manual", "auto"];
const USERNAME_MAX: usize = 64;

fn bad(msg: &str) -> Result<(), ApiError> {
    Err((StatusCode::BAD_REQUEST, msg.to_string()))
}

// Letters/digits plus . _ - ; 1..=64 chars. Applies to new accounts only.
pub fn username(name: &str) -> Result<(), ApiError> {
    let n = name.trim();
    if n.is_empty() {
        return bad("El usuario no puede estar vacío");
    }
    if n.chars().count() > USERNAME_MAX {
        return bad("El usuario es demasiado largo");
    }
    if !n
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return bad("El usuario solo admite letras, números y . _ -");
    }
    Ok(())
}

// ISO-4217-ish: exactly 3 ASCII letters (e.g. USD, MXN). Case left as sent.
pub fn currency(code: &str) -> Result<(), ApiError> {
    if code.len() == 3 && code.bytes().all(|b| b.is_ascii_alphabetic()) {
        Ok(())
    } else {
        bad("Moneda inválida (código ISO de 3 letras)")
    }
}

pub fn days_before(n: i64) -> Result<(), ApiError> {
    if n >= 0 {
        Ok(())
    } else {
        bad("days_before inválido")
    }
}

pub fn amount(cents: i64) -> Result<(), ApiError> {
    if cents >= 0 {
        Ok(())
    } else {
        bad("El importe no puede ser negativo")
    }
}

pub fn cycle(c: &str) -> Result<(), ApiError> {
    if CYCLES.contains(&c) {
        Ok(())
    } else {
        bad("Ciclo inválido")
    }
}

pub fn status(s: &str) -> Result<(), ApiError> {
    if STATUSES.contains(&s) {
        Ok(())
    } else {
        bad("Estado inválido")
    }
}

pub fn payment_mode(p: &str) -> Result<(), ApiError> {
    if PAYMENT_MODES.contains(&p) {
        Ok(())
    } else {
        bad("Modo de pago inválido")
    }
}

// Full check for a complete row (create + import). `status` is the effective
// value (create passes "active").
pub fn subscription(
    amount_cents: i64,
    currency_code: &str,
    cycle_kind: &str,
    cycle_days: Option<i64>,
    mode: &str,
    state: &str,
) -> Result<(), ApiError> {
    amount(amount_cents)?;
    currency(currency_code)?;
    cycle(cycle_kind)?;
    if cycle_kind == "custom" && cycle_days.is_none_or(|d| d < 1) {
        return bad("Un ciclo personalizado requiere cycle_days >= 1");
    }
    payment_mode(mode)?;
    status(state)
}
