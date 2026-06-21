<div align="center">

# Dueo

**Panel autoalojado para no perder de vista tus suscripciones y pagos recurrentes.**

Cada servicio es un anillo de progreso que avanza desde que lo contrataste hasta su
vencimiento, y te avisa —en la web, por Telegram o por correo— antes de que se acabe.

`Rust + Axum` · `SvelteKit (Svelte 5)` · `SQLite` · **un solo binario**

![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)
![Svelte 5](https://img.shields.io/badge/Svelte-5-ff3e00.svg)

[Arquitectura y decisiones →](docs/ARCHITECTURE.md) · [Arrancar](#arrancar-con-docker) · [Desarrollo](#desarrollo)

</div>

---

Pagamos cada vez más servicios recurrentes (streaming, dominios, hosting, VPS,
licencias, seguros…). El problema no es *cuánto* gastamos, sino **perder de vista
cuándo vence cada cosa** —sobre todo si no domicilias los pagos y dependes de tu
memoria. Dueo lo vuelve visible de un vistazo y te recuerda a tiempo.

Tus datos viven en **tu** servidor: SQLite en un archivo, sin servicios externos ni
telemetría.

## Características

- **Progreso visual ("Horizonte").** Línea de tiempo viva con todos tus servicios y
  su urgencia por proximidad; cada fila muestra cuánta "vida" le queda al servicio.
- **Recordatorios configurables.** Avisa N días antes (1/2/3/5/7… o lo que pongas),
  con reglas globales o propias por servicio, en la **zona horaria** y a la **hora**
  de cada usuario.
- **Tres canales de aviso.** In-app en tiempo real (SSE, sin recargar), **Telegram** y
  **correo (SMTP)** — y los mensajes llegan en el **idioma** del usuario.
- **Recurrencia automática.** Las domiciliadas se auto-renuevan al vencer; las
  manuales pasan a *vencidas* para que las renueves tú.
- **Multi-usuario real.** Varias cuentas en una instancia con **aislamiento estricto**
  de datos y rol de administrador para gestionarlas.
- **Categorías, iconos y color por servicio** (catálogo de marcas cargado bajo
  demanda) y **multi-moneda** (agrupada por divisa, mostrada con código ISO).
- **Insights.** Gasto por categoría, top de servicios y proyección de caja a 6 meses.
- **Bilingüe (ES/EN)** y preparado para más idiomas; tema claro/oscuro.
- **Backup integrado.** Export/import de tus datos en JSON desde la propia app.
- **Autoalojado de un comando.** Un binario con el front embebido, o `docker compose up`.

<!-- Capturas: añade aquí imágenes de docs/ (dashboard, horizonte, insights). -->

## Arrancar con Docker

Requiere Docker con Compose.

```bash
git clone https://github.com/Yelt-dev/dueo.git
cd dueo
docker compose up -d
```

Abre **http://localhost:3000**. En el primer arranque la instancia está vacía y la
pantalla de acceso te pide **crear la cuenta de administrador** (el primer usuario
registrado es admin; después el registro queda cerrado salvo que lo abras —ver más
abajo). La base SQLite se guarda en el volumen `dueo-data` y persiste entre reinicios.

El `docker-compose.yml` usa la imagen publicada `ghcr.io/yelt-dev/dueo:latest`. Para
compilar desde el código en su lugar, descomenta `build: .` en el compose y arranca con
`docker compose up -d --build` (ver «Compilar desde el código»).

## Actualizar

Tu base de datos vive en el volumen `dueo-data`, **independiente de la imagen**, así que
actualizar **no afecta tus datos**: las migraciones de esquema se aplican solas al
arrancar. Para pasar a la última versión:

```bash
docker compose pull && docker compose up -d
```

Esto baja la última imagen publicada, recrea el contenedor y conserva todo (datos +
configuración). Para fijar una versión concreta en vez de seguir `:latest`, cambia el tag
en el compose (p. ej. `ghcr.io/yelt-dev/dueo:0.2`). Si prefieres no intervenir, un sidecar
tipo [Watchtower](https://containrrr.dev/watchtower/) puede vigilar `:latest` y actualizar
solo.

## Configuración

Todo se configura por variables de entorno (o un archivo `.env` junto al binario; ver
`dueo-server/.env.example`).

| Variable | Por defecto | Para qué |
| --- | --- | --- |
| `DUEO_BIND` | `127.0.0.1:3000` | dirección y puerto de escucha (`0.0.0.0:3000` en contenedor) |
| `DUEO_OPEN_REGISTRATION` | `0` | permitir auto-registro tras crear el admin |
| `DUEO_SECURE_COOKIE` | `0` | marcar la cookie de sesión como `Secure` (tras HTTPS) |
| `DUEO_TELEGRAM_BOT_TOKEN` | *(vacío)* | activa el canal Telegram |
| `DUEO_SMTP_HOST` / `PORT` / `USER` / `PASS` / `FROM` | *(vacío)* | activa el canal email (SMTP, STARTTLS) |

Los canales son opcionales: sin token de Telegram o sin SMTP, el resto de la app
funciona igual. El **destino** de cada canal (chat de Telegram, correo) lo configura
cada usuario desde **Ajustes**, porque es multi-cuenta.

## Copias de seguridad

- **Desde la app:** Ajustes → Datos → *Exportar* descarga un `.json`; *Importar* lo
  vuelve a cargar.
- **A nivel de instancia:** copia el archivo `dueo.db` (en Docker, el volumen
  `dueo-data`).

## Compilar desde el código

Requisitos: Rust (edición 2024, ≥ 1.85) y Node con pnpm.

```bash
# 1) front estático → dueo-web/build/
cd dueo-web && pnpm install && pnpm build && cd ..

# 2) binario release (embebe dueo-web/build)
cd dueo-server && cargo build --release   # → target/release/dueo-server
```

El binario resultante es **autónomo**: lleva el front embebido y las migraciones
incluidas. El orden importa: el front debe estar construido antes de compilar en release.

## Desarrollo

Front y back por separado, con proxy de Vite (`/api` → `:3000`):

```bash
cd dueo-server && cargo run      # backend
cd dueo-web && pnpm dev          # frontend → http://localhost:5173
```

En debug el binario lee el front de `dueo-web/build/` desde disco; en release lo embebe.

## Arquitectura

El diseño, las decisiones técnicas y los trade-offs están documentados en
**[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)**: aislamiento multi-usuario, el motor
de recordatorios idempotente, notificaciones en tiempo real, recurrencia, seguridad y
la distribución de un solo binario.

## Licencia

[MIT](LICENSE) © Yeltsin López
