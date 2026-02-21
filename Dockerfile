# check=skip=SecretsUsedInArgOrEnv
# Stage 1: Cargo Chef Base
FROM rust:1.91-slim AS chef
WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev make \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked

# Stage 2: Recipe Planner
FROM chef AS planner

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src/ ./src/

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Rust Builder
FROM chef AS builder

# Cook dependencies (cached until Cargo.toml/Cargo.lock change)
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source, migrations, and SQLx offline cache
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src/ ./src/
COPY backend/migrations/ ./migrations/
COPY backend/views.sql ./views.sql
COPY backend/.sqlx/ ./.sqlx/

# Build with SQLx offline mode (no live database needed)
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Stage 4: Frontend Builder
FROM oven/bun:1 AS frontend
WORKDIR /build

# Install dependencies (standalone, no workspace)
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile

COPY frontend/ ./

# PostHog source map upload (optional: set via --build-arg in Railway)
ARG POSTHOG_PERSONAL_API_KEY
ARG POSTHOG_PROJECT_ID

RUN bun --smol run build

# Stage 5: Runtime
FROM oven/bun:1-slim
WORKDIR /app

# Install runtime dependencies (ca-certificates for HTTPS, wget for healthcheck)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates wget \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust binary
COPY --from=builder /build/target/release/glint ./glint

# Copy SvelteKit build output
COPY --from=frontend /build/build ./web/build

# Copy production node_modules (runtime dependencies externalized by the adapter)
COPY --from=frontend /build/node_modules ./web/node_modules

# Copy entrypoint and console logger (preloaded by Bun to intercept stray console.* calls)
COPY frontend/entrypoint.ts ./web/entrypoint.ts
COPY frontend/console-logger.js ./web/console-logger.js

# Environment defaults
# PORT = public-facing SvelteKit (Railway injects $PORT)
# GLINT_PORT = internal Axum backend (not exposed)
# LOG_JSON = structured JSON output (both backend and frontend read this)
ENV PORT=8080 \
    GLINT_HOST=127.0.0.1 \
    GLINT_PORT=3001 \
    BACKEND_URL=http://localhost:3001 \
    LOG_JSON=true \
    TZ=Etc/UTC

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
    CMD wget -q --spider http://localhost:${PORT}/api/health || exit 1

ENTRYPOINT ["bun", "run", "/app/web/entrypoint.ts"]
