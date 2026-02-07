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

# Run pending migrations
migrate-run:
    cd backend && cargo sqlx migrate run

# Create new migration file
migrate-create name:
    cd backend && cargo sqlx migrate add {{name}}

# Seed database with sample data
db-seed:
    cargo run --manifest-path backend/Cargo.toml --quiet -- seed

# === Utilities ===

# Regenerate TypeScript bindings from Rust types
bindings:
    cd backend && cargo test export_bindings --quiet
    bun scripts/bindings-barrel.ts

# Generate optimized wallpapers and thumbhash manifest
wallpapers:
    bun scripts/optimize-wallpapers.ts

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

# Install git pre-commit hooks
install-hooks:
    #!/usr/bin/env bash
    set -euo pipefail
    chmod +x scripts/pre-commit.ts
    echo "bun scripts/pre-commit.ts" > .husky/pre-commit
    chmod +x .husky/pre-commit
    echo "✓ Pre-commit hook installed"
