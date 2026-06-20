// Declaramos los módulos (cada uno es su archivo .rs).
mod auth;
mod categories;
mod channels;
mod data;
mod email;
mod notifications;
mod reminders;
mod scheduler;
mod subscriptions;
mod telegram;
mod users;
mod validate;

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rust_embed::RustEmbed;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Evento de notificación nueva (in-app) para empujar por SSE en vivo.
// `json` ya es la notificación serializada lista para el cliente.
#[derive(Clone)]
pub struct NotifEvent {
    pub user_id: i64,
    pub json: String,
}

// Contador de intentos fallidos de login por usuario, en memoria (no necesita
// persistencia: si el proceso reinicia, se limpia). (fallos, inicio de ventana).
pub type LoginAttempts = Arc<Mutex<HashMap<String, (u32, Instant)>>>;

// Estado compartido por todos los handlers. `pub` para que los módulos lo usen.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub tx: tokio::sync::broadcast::Sender<NotifEvent>,
    pub login_attempts: LoginAttempts,
}

// Tipo de error de la API y helper, compartidos por todos los módulos.
pub type ApiError = (StatusCode, String);

// Log the real error server-side; never leak its Display (SQL fragments, paths)
// into the HTTP response body.
pub fn internal<E: std::fmt::Display>(e: E) -> ApiError {
    eprintln!("[error] {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Error interno".to_string(),
    )
}

// Map only a UNIQUE-constraint violation to 409 `msg`; any other DB error is a 500.
pub fn unique_or_internal(msg: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| match e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            (StatusCode::CONFLICT, msg.to_string())
        }
        other => internal(other),
    }
}

// Lector mínimo de `.env` (sin crate): KEY=VALUE por línea. Se llama al arrancar,
// antes de cualquier hilo, así que `set_var` es seguro.
fn load_dotenv() {
    let Ok(content) = std::fs::read_to_string(".env") else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().trim_matches('"').trim_matches('\'');
            unsafe { std::env::set_var(k.trim(), v) };
        }
    }
}

#[tokio::main]
async fn main() {
    load_dotenv(); // carga el token de Telegram, etc. (si existe `.env`)

    // foreign_keys(true): SQLite NO aplica las FK por defecto. Lo activamos para
    // que ON DELETE CASCADE funcione (borrar un usuario arrastra sus datos) y
    // ON DELETE SET NULL también. Es por-conexión, por eso va en las opciones del pool.
    let opts = SqliteConnectOptions::from_str("sqlite:dueo.db?mode=rwc")
        .unwrap()
        .foreign_keys(true);
    let db = SqlitePool::connect_with(opts).await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();

    // Canal broadcast para empujar notificaciones nuevas por SSE.
    let (tx, _rx) = tokio::sync::broadcast::channel::<NotifEvent>(256);
    let state = AppState {
        db,
        tx,
        login_attempts: Arc::new(Mutex::new(HashMap::new())),
    };

    // Scheduler de recordatorios en segundo plano (cada hora, por zona del usuario).
    tokio::spawn(scheduler::run_loop(state.clone()));
    // Limpieza periódica de sesiones caducadas.
    tokio::spawn(auth::session_cleanup_loop(state.clone()));

    // Todas las rutas de la API viven bajo /api.
    let api = Router::new()
        .route("/health", get(health))
        // auth
        .route("/setup-status", get(auth::setup_status))
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/me", get(auth::me))
        .route("/settings", axum::routing::put(auth::update_settings))
        // seguridad: cambiar contraseña y cerrar todas las sesiones
        .route("/password", post(auth::change_password))
        .route("/logout-all", post(auth::logout_all))
        // datos: export/backup e import
        .route("/export", get(data::export))
        .route("/import", post(data::import))
        // usuarios (solo admin): listar, crear, borrar
        .route("/users", get(users::list).post(users::create))
        .route("/users/{id}", axum::routing::delete(users::delete))
        // acerca de: nombre + versión de la instancia (público)
        .route("/version", get(version))
        // subscriptions (CRUD)
        .route(
            "/subscriptions",
            get(subscriptions::list).post(subscriptions::create),
        )
        .route(
            "/subscriptions/{id}",
            get(subscriptions::get_one)
                .patch(subscriptions::update)
                .delete(subscriptions::delete),
        )
        // categories (CRUD)
        .route(
            "/categories",
            get(categories::list).post(categories::create),
        )
        .route(
            "/categories/{id}",
            axum::routing::patch(categories::update).delete(categories::delete),
        )
        // reminders (reglas de anticipación)
        .route("/reminders", get(reminders::list).post(reminders::create))
        .route("/reminders/{id}", axum::routing::delete(reminders::delete))
        // notifications (panel in-app)
        .route("/notifications", get(notifications::list))
        .route("/notifications/stream", get(notifications::stream))
        .route("/notifications/read", post(notifications::mark_all_read))
        .route("/notifications/{id}/read", post(notifications::mark_read))
        // scheduler (disparo manual para dev/test)
        .route("/scheduler/run", post(scheduler::run_now))
        // telegram (canal): estado, configurar destino, enviar prueba
        .route(
            "/channels/telegram",
            get(telegram::status).put(telegram::set_config),
        )
        .route("/channels/telegram/test", post(telegram::test_send))
        // email (canal): estado, configurar destino, enviar prueba
        .route("/channels/email", get(email::status).put(email::set_config))
        .route("/channels/email/test", post(email::test_send));

    // El front (SvelteKit estático) se sirve desde el propio binario: la API va
    // bajo /api y CUALQUIER otra ruta cae en el handler estático (SPA fallback).
    let app = Router::new()
        .nest("/api", api)
        .fallback(static_handler)
        .layer(middleware::from_fn(security_headers))
        .with_state(state);

    // Dirección de escucha configurable. Local: 127.0.0.1:3000 (default). En
    // contenedor hay que escuchar en 0.0.0.0 para ser accesible: DUEO_BIND=0.0.0.0:3000.
    let bind = std::env::var("DUEO_BIND").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    println!("Dueo escuchando en http://{bind}");
    axum::serve(listener, app).await.unwrap();
}

// Front embebido. En debug rust-embed lee de disco (no hace falta recompilar al
// cambiar el front); en release lo embebe en el binario → un solo ejecutable.
// La carpeta es relativa al crate; el front debe estar construido (pnpm build).
#[derive(RustEmbed)]
#[folder = "../dueo-web/build/"]
struct WebAssets;

// Sirve el archivo embebido para la ruta pedida; si no existe (ruta de SPA como
// /ajustes), devuelve index.html para que el router del cliente la resuelva.
async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = WebAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response();
    }

    // Fallback SPA: rutas del cliente sin archivo propio → index.html.
    match WebAssets::get("index.html") {
        Some(index) => (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            index.data,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "Front no embebido. Ejecuta `pnpm build` en dueo-web.",
        )
            .into_response(),
    }
}

// Acerca de: nombre y versión del binario (desde Cargo.toml en tiempo de compilación).
async fn version() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// Conservative security headers on every response. No CSP (the SPA uses inline
// styles/scripts); session cookie is SameSite=Lax, which already blocks the
// cross-site requests that would carry CSRF on mutating routes.
async fn security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    h.insert("Referrer-Policy", HeaderValue::from_static("same-origin"));
    res
}

async fn health(State(state): State<AppState>) -> String {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap();
    format!("OK — usuarios registrados: {}", count.0)
}
