-- Migración 0002: sesiones. Cada login crea una fila aquí.
-- `id` es el token aleatorio que viaja en la cookie del navegador.
-- ON DELETE CASCADE: si se borra el usuario, sus sesiones se borran solas.

CREATE TABLE sessions (
    id         TEXT    PRIMARY KEY,                 -- token opaco aleatorio
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
