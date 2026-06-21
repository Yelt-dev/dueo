# Changelog

Formato basado en [Keep a Changelog](https://keepachangelog.com/es/1.1.0/);
versionado [SemVer](https://semver.org/lang/es/).

## [0.2.0](https://github.com/Yelt-dev/dueo/compare/v0.1.0...v0.2.0) (2026-06-21)


### Features

* **web:** typed API client, Popover primitive, settings & modal refactor ([2bc6c9c](https://github.com/Yelt-dev/dueo/commit/2bc6c9c073ff42b587bcb68fdcbc2e08f8c5847f))

## [0.1.0] — 2026-06-20

Primera versión pública. Panel autoalojado de suscripciones y pagos recurrentes,
distribuido como un solo binario (front SvelteKit embebido en el servidor Rust).

### Funcionalidad
- CRUD de suscripciones y categorías, con icono y color por servicio (catálogo de
  marcas cargado bajo demanda).
- Vista **Horizonte**: línea de tiempo con la urgencia de cada servicio por proximidad.
- **Recordatorios** configurables (N días antes, globales o por servicio) con motor
  horario idempotente, en la zona horaria y hora de cada usuario.
- Tres canales de aviso: **in-app** en tiempo real (SSE), **Telegram** y **correo (SMTP)**.
- Recurrencia automática (domiciliadas se auto-renuevan; manuales pasan a vencidas).
- **Multi-usuario** con aislamiento estricto por usuario y rol de administrador.
- **Insights**: gasto por categoría, top de servicios y proyección de caja a 6 meses.
- Multi-moneda agrupada por divisa (código ISO), sin conversión.
- Bilingüe **ES/EN** (UI), tema claro/oscuro, export/import de datos en JSON.

### Distribución y seguridad
- Imagen Docker de un comando (`docker compose up -d --build`) e instrucciones para
  compilar el binario autónomo.
- Registro cerrado por defecto, rate-limit de login, contraseñas Argon2, sesiones
  revocables, cabeceras de seguridad y cookie `SameSite=Lax`.

[0.1.0]: https://github.com/Yelt-dev/dueo/releases/tag/v0.1.0
