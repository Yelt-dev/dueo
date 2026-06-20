-- Migración 0001: tabla de usuarios (raíz del multi-usuario).
-- Cada cuenta de la instancia selfhosted es una fila aquí.

CREATE TABLE users (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    username         TEXT    NOT NULL UNIQUE,
    password_hash    TEXT    NOT NULL,                 -- Argon2 (nunca la contraseña en claro)
    role             TEXT    NOT NULL DEFAULT 'member', -- 'admin' | 'member'
    timezone         TEXT    NOT NULL DEFAULT 'UTC',
    default_currency TEXT    NOT NULL DEFAULT 'USD',
    created_at       TEXT    NOT NULL DEFAULT (datetime('now'))  -- UTC ISO-8601
);
