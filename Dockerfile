# Dueo — imagen de un solo binario (front embebido) para selfhosting.
# Multi-stage: 1) construye el front, 2) compila el server embebiéndolo,
# 3) imagen runtime mínima con solo el binario.

# --- Stage 1: build del front (SvelteKit → estático) -----------------------
FROM node:22-slim AS web
RUN corepack enable && corepack prepare pnpm@10.32.1 --activate
WORKDIR /app/dueo-web
# Primero el manifiesto + lockfile: capa cacheable de dependencias.
COPY dueo-web/package.json dueo-web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY dueo-web/ ./
RUN pnpm build   # genera dueo-web/build/ (index.html + _app/…)

# --- Stage 2: build del server (rust-embed embebe dueo-web/build) ----------
FROM rust:1.96-slim AS server
# build-essential: libsqlite3-sys compila SQLite (C) al hacer el build.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY dueo-server/ ./dueo-server/
# El crate embebe "../dueo-web/build/": lo dejamos donde lo espera (junto al crate).
COPY --from=web /app/dueo-web/build ./dueo-web/build
WORKDIR /app/dueo-server
RUN cargo build --release

# --- Stage 3: runtime mínimo -----------------------------------------------
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# /data guarda la base SQLite (se monta como volumen para que persista).
WORKDIR /data
COPY --from=server /app/dueo-server/target/release/dueo-server /usr/local/bin/dueo-server
# En contenedor hay que escuchar en todas las interfaces.
ENV DUEO_BIND=0.0.0.0:3000
EXPOSE 3000
CMD ["dueo-server"]
