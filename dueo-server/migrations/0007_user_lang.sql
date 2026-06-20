-- Migración 0007: idioma preferido del usuario para los MENSAJES del servidor
-- (recordatorios in-app/Telegram/email). La UI usa su propio i18n en el cliente;
-- esto solo afecta al texto que GENERA el backend. 'es' | 'en'.
ALTER TABLE users ADD COLUMN lang TEXT NOT NULL DEFAULT 'es';
