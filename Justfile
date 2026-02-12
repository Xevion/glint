# Glint - Shader Preview Catalog

set positional-arguments := true

# === Aliases ===

# Core development
alias c := check
alias d := dev
alias t := test
alias f := format

# Subsystem shortcuts
alias cm := check-mod
alias dm := dev-mod
alias sm := smoke
alias lf := loom-fix

# === Default ===

default:
    @just --list

# === Core Development ===

# Validate all code (parallel checks, auto-fix formatting when safe)
check *flags:
    bun scripts/check.ts {{flags}}

# Dev server - multi-subsystem orchestration. Flags: -f(rontend) -b(ackend) -m(od) -W(no-watch) -r(elease) -p(latform) -- <args>
dev *flags:
    bun scripts/dev.ts {{flags}}

# Run all unit tests (parallel). Usage: just test [web|web-e2e|rust|mod|<nextest filter>]
test *args:
    bun scripts/test.ts {{args}}

# Run E2E tests (Playwright)
test-e2e:
    bun run --cwd frontend test:e2e

# Auto-format all code
format:
    bun run --cwd frontend format
    cargo fmt --manifest-path backend/Cargo.toml
    cd mod && ./gradlew spotlessApply ktlintFormat --quiet

# Lint all code
lint:
    bun run --cwd frontend lint
    cargo clippy --manifest-path backend/Cargo.toml -- --deny warnings
    cd mod && ./gradlew detekt --quiet

# Build everything for production
build:
    bun run --cwd frontend build
    cargo build --manifest-path backend/Cargo.toml --release
    cd mod && ./gradlew build --quiet

# === Frontend ===

# Run bun commands in frontend (e.g., `just bun run test`)
bun *args:
    cd frontend && bun {{args}}

# === Backend ===
# (Use `just dev -b` for backend dev server)

# === Mod Development ===

# Run Minecraft client (Fabric by default)
dev-mod platform="fabric":
    bun scripts/dev.ts -m -p {{platform}}

# Quick compile check for mod
check-mod:
    cd mod && ./gradlew :common:compileKotlin :common:compileJava --quiet

# Integration test - verify client starts and mixins load
smoke platform="fabric":
    bun ./scripts/smoke.ts {{platform}}

# Run autonomous shader capture orchestration
orchestrate *flags:
    bun scripts/orchestrate.ts {{flags}}

# Fix Loom remapping cache corruption (stale intermediary names in mixin annotations)
loom-fix:
    rm -rf mod/.gradle/loom-cache/remapped_mods
    cd mod && ./gradlew --stop
    cd mod && ./gradlew :fabric:configureClientLaunch --refresh-dependencies
    @echo "✓ Loom remapped_mods cache rebuilt"

# Run any command in mod directory (e.g., `just mod ./gradlew clean`)
mod *args:
    cd mod && {{args}}

# === Database ===

# Start PostgreSQL in Docker and update .env with connection string
# Commands: start (default), reset, rm
db cmd="start":
    bun scripts/db.ts {{cmd}}

# Reset database (drops and recreates)
migrate-reset:
    cd backend && cargo sqlx database reset -y

# Run pending migrations, then apply views
migrate-run:
    cd backend && cargo sqlx migrate run
    @echo "Applying views..."
    cd backend && bash -c 'source .env 2>/dev/null && psql "$DATABASE_URL" -f views.sql -q' || echo "⚠ psql not available or failed; views will be applied on next app startup"

# Create new migration file
migrate-create name:
    cd backend && cargo sqlx migrate add {{name}}

# Seed database with sample data
db-seed:
    cargo run --manifest-path backend/Cargo.toml --quiet -- seed

# === R2 / S3 ===

# Run aws s3 commands against the R2 bucket.
# Usage: just r2 ls worlds/ | just r2 cp R2/worlds/a.zip R2/worlds/b.zip
# Shorthand: R2 or R2/ expands to s3://<bucket> or s3://<bucket>/
r2 *args:
    #!/usr/bin/env bash
    set -euo pipefail
    [ -f .env ] && set -a && source .env && set +a
    export AWS_ACCESS_KEY_ID="$GLINT_R2_ACCESS_KEY_ID"
    export AWS_SECRET_ACCESS_KEY="$GLINT_R2_SECRET_ACCESS_KEY"
    export AWS_ENDPOINT_URL="https://${GLINT_R2_ACCOUNT_ID}.r2.cloudflarestorage.com"
    bucket="s3://${GLINT_R2_BUCKET}"
    # Allow shorthand: R2 or R2/ expands to s3://<bucket> or s3://<bucket>/
    args=()
    for arg in "$@"; do
        if [[ "$arg" == "R2" ]]; then
            args+=("$bucket")
        elif [[ "$arg" == R2/* ]]; then
            args+=("${bucket}/${arg#R2/}")
        else
            args+=("$arg")
        fi
    done
    exec aws s3 "${args[@]}"

# Alias: just s3 → just r2
alias s3 := r2

# === Utilities ===

# Regenerate TypeScript bindings from Rust types
bindings:
    cd backend && cargo test export_bindings --quiet
    bun scripts/bindings-barrel.ts

# Query Minecraft source JAR
# Usage:
#   just mcjar list net/minecraft/client/renderer/        # List classes in package
#   just mcjar cat net/minecraft/client/Minecraft.java    # Read entire class
#   just mcjar grep shouldEntityAppearGlowing net/minecraft/client/Minecraft.java  # Search in class
#   just mcjar grep-all startUseItem 'net/minecraft/client/*.java'  # Search multiple files
#   just mcjar asset rendertype_lines.vsh  # Read shader/asset file
#   just mcjar asset-list shaders/  # List asset files
mcjar +args='':
    #!/usr/bin/env bash
    set -euo pipefail
    exec bun ./scripts/mcjar.ts "$@"

# === Docker (Web Server) ===

# Build the web server Docker image
web-build *flags:
    docker build -t glint-web:latest {{flags}} .

# Run the web server in Docker (reads backend/.env for DATABASE_URL and credentials)
web-run *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f backend/.env ]; then
        echo "ERROR: backend/.env not found" >&2
        exit 1
    fi
    docker run --rm -it \
        --network host \
        --env-file backend/.env \
        {{flags}} \
        glint-web:latest

# === Docker (Headless Capture) ===

# Build the headless capture Docker image
capture-build *flags:
    docker build -t glint-capture:latest -f docker/Dockerfile {{flags}} .

# Run a capture session in Docker (requires NVIDIA GPU + Container Toolkit)
# Set RECORD=true to save a screen recording to docker/output/capture.mp4
capture-run *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    [ -f .env ] && set -a && source .env && set +a
    # NVIDIA Container Toolkit (as of early 2026) doesn't mount libnvidia-gpucomp.so,
    # a dependency added in driver ~560+. Find and mount it if present on the host.
    GPUCOMP_MOUNT=""
    GPUCOMP_PATH=$(find /usr/lib/x86_64-linux-gnu -name "libnvidia-gpucomp.so.*" -not -name "*.so" 2>/dev/null | head -1 || true)
    if [ -n "$GPUCOMP_PATH" ]; then
        GPUCOMP_MOUNT="-v ${GPUCOMP_PATH}:${GPUCOMP_PATH}:ro"
    fi
    mkdir -p docker/output
    docker run --rm \
        --gpus all \
        --device=/dev/dri:/dev/dri \
        --add-host=host.docker.internal:host-gateway \
        -e NVIDIA_DRIVER_CAPABILITIES=all \
        -e GLINT_AUTONOMOUS="${GLINT_AUTONOMOUS:-true}" \
        -e GLINT_API_URL="${GLINT_API_URL:-http://host.docker.internal:8080}" \
        -e GLINT_API_TOKEN="${GLINT_API_TOKEN:?GLINT_API_TOKEN must be set in .env}" \
        -e RECORD="${RECORD:-false}" \
        -v mc-assets:/minecraft/assets \
        -v "$(pwd)/docker/output:/output" \
        $GPUCOMP_MOUNT \
        {{flags}} \
        glint-capture:latest

# Smoke test: verify Xvfb + VirtualGL can see the GPU inside the container
capture-smoke:
    #!/usr/bin/env bash
    set -euo pipefail
    GPUCOMP_MOUNT=""
    GPUCOMP_PATH=$(find /usr/lib/x86_64-linux-gnu -name "libnvidia-gpucomp.so.*" -not -name "*.so" 2>/dev/null | head -1 || true)
    if [ -n "$GPUCOMP_PATH" ]; then
        GPUCOMP_MOUNT="-v ${GPUCOMP_PATH}:${GPUCOMP_PATH}:ro"
    fi
    docker run --rm \
        --gpus all \
        --device=/dev/dri:/dev/dri \
        -e NVIDIA_DRIVER_CAPABILITIES=all \
        $GPUCOMP_MOUNT \
        glint-capture:latest \
        bash -c 'Xvfb :99 -screen 0 1024x768x24 -ac +iglx &>/dev/null & sleep 2 && DISPLAY=:99 VGL_DISPLAY=egl vglrun glxinfo | head -30'

# Install git pre-commit hooks
install-hooks:
    #!/usr/bin/env bash
    set -euo pipefail
    chmod +x scripts/pre-commit.ts
    echo "bun scripts/pre-commit.ts" > .husky/pre-commit
    chmod +x .husky/pre-commit
    echo "✓ Pre-commit hook installed"
