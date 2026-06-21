# Dueo — single-binary image (front embedded) for self-hosting.
# Multi-stage: 1) build the front, 2) compile the server embedding it,
# 3) minimal runtime image with just the binary.

# --- Stage 1: build the front (SvelteKit → static) -------------------------
FROM node:22-slim AS web
RUN corepack enable && corepack prepare pnpm@10.32.1 --activate
WORKDIR /app/dueo-web
# Manifest + lockfile first: cacheable dependency layer.
COPY dueo-web/package.json dueo-web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY dueo-web/ ./
RUN pnpm build   # produces dueo-web/build/ (index.html + _app/…)

# --- Stage 2: build the server (rust-embed embeds dueo-web/build) ----------
FROM rust:1.96-slim AS server
# build-essential: libsqlite3-sys compiles SQLite (C) during the build.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY dueo-server/ ./dueo-server/
# The crate embeds "../dueo-web/build/": place it where it's expected (next to the crate).
COPY --from=web /app/dueo-web/build ./dueo-web/build
WORKDIR /app/dueo-server
RUN cargo build --release

# --- Stage 3: minimal runtime ----------------------------------------------
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# /data holds the SQLite DB (mounted as a volume so it persists).
WORKDIR /data
COPY --from=server /app/dueo-server/target/release/dueo-server /usr/local/bin/dueo-server
# In a container we must listen on all interfaces.
ENV DUEO_BIND=0.0.0.0:3000
EXPOSE 3000
CMD ["dueo-server"]
