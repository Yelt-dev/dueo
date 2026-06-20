-- Migración 0003: categorías y suscripciones (el dominio principal).
-- Ambas cuelgan de user_id: cada usuario tiene SOLO sus datos (aislamiento R13.1).

CREATE TABLE categories (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT    NOT NULL,
    color      TEXT,
    icon       TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_categories_user ON categories(user_id);

CREATE TABLE subscriptions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    amount_cents INTEGER NOT NULL DEFAULT 0,            -- dinero SIEMPRE en centavos (R1)
    currency     TEXT    NOT NULL DEFAULT 'USD',        -- ISO 4217
    cycle        TEXT    NOT NULL DEFAULT 'monthly',    -- monthly|yearly|once|custom
    cycle_days   INTEGER,                               -- solo si cycle = custom
    start_date   TEXT    NOT NULL,                      -- fecha contratación (UTC, YYYY-MM-DD)
    due_date     TEXT    NOT NULL,                      -- vencimiento actual
    category_id  INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    payment_mode TEXT    NOT NULL DEFAULT 'manual',     -- auto|manual
    status       TEXT    NOT NULL DEFAULT 'active',     -- active|paused|expired|cancelled|archived
    notes        TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_subscriptions_user ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_due  ON subscriptions(due_date);
