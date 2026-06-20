-- Migración 0005: hora local de aviso diario por usuario (0-23).
-- `timezone` ya existe desde 0001; aquí solo añadimos send_hour.
-- Los recordatorios se evalúan en la zona del usuario a su hora de aviso.
ALTER TABLE users ADD COLUMN send_hour INTEGER NOT NULL DEFAULT 9;
