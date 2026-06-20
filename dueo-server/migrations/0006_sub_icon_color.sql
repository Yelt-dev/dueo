-- Migración 0006: icono y color personalizables por suscripción.
-- icon  = id del catálogo (p.ej. 'si:netflix' Simple Icons, 'lu:tv' Lucide). NULL = autodetectar por nombre.
-- color = tono de la marca Dueo (hsl/hex). NULL = usar el de la marca/categoría.
ALTER TABLE subscriptions ADD COLUMN icon  TEXT;
ALTER TABLE subscriptions ADD COLUMN color TEXT;
