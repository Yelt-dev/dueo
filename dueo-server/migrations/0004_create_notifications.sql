-- Migración 0004: motor de notificaciones (reglas, canales, log).
-- Todo cuelga de user_id (aislamiento R13.1). Las reglas globales del usuario
-- son las que tienen subscription_id NULL (default por usuario, no de la instancia).

-- Anticipaciones: "avisar N días antes del vencimiento".
-- subscription_id NULL = regla GLOBAL del usuario (default); con valor = por servicio.
CREATE TABLE reminder_rules (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_id INTEGER REFERENCES subscriptions(id) ON DELETE CASCADE,
    days_before     INTEGER NOT NULL,                    -- 1,2,3,5,7…
    -- evita reglas duplicadas (misma anticipación para el mismo ámbito)
    UNIQUE(user_id, subscription_id, days_before)
);
CREATE INDEX idx_reminder_rules_user ON reminder_rules(user_id);

-- Configuración por canal y usuario (p.ej. el chat_id de Telegram).
CREATE TABLE channel_config (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind        TEXT    NOT NULL,                        -- 'inapp' | 'telegram' | …
    enabled     INTEGER NOT NULL DEFAULT 1,              -- bool
    config_json TEXT    NOT NULL DEFAULT '{}',           -- p.ej. {"chat_id":"..."}
    UNIQUE(user_id, kind)
);
CREATE INDEX idx_channel_config_user ON channel_config(user_id);

-- Log de notificaciones: da IDEMPOTENCIA (R10) y alimenta el panel in-app.
-- El UNIQUE garantiza que cada (servicio, canal, fecha objetivo, anticipación)
-- se dispare UNA sola vez aunque el scheduler corra varias veces.
CREATE TABLE notification_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_id INTEGER NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    channel         TEXT    NOT NULL,                    -- 'inapp' | 'telegram'
    target_due_date TEXT    NOT NULL,                    -- la due_date a la que apuntaba
    days_before     INTEGER NOT NULL,                    -- qué anticipación lo disparó
    message         TEXT    NOT NULL,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    sent_at         TEXT,                                -- NULL = pendiente / falló el envío
    read_at         TEXT,                                -- para in-app (marcar leído)
    UNIQUE(subscription_id, channel, target_due_date, days_before)
);
CREATE INDEX idx_notification_log_user ON notification_log(user_id, created_at);
