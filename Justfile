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

# Run all unit tests (parallel). Usage: just test [web|web-e2e|rust|mod|agent|<nextest filter>]
test *args:
    bun scripts/test.ts {{args}}

# Run E2E tests (Playwright)
test-e2e:
    bun run --cwd frontend test:e2e

# Auto-format all code
format:
    bun run --cwd frontend format
    cargo fmt --manifest-path backend/Cargo.toml
    cargo fmt --manifest-path agent/Cargo.toml
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

# Start frontend dev server
dev-fe:
    bun scripts/dev.ts -f

# Run any command in frontend directory
fe *args:
    cd frontend && {{args}}

# Run bun commands in frontend (e.g., `just bun run test`)
bun *args:
    cd frontend && bun {{args}}

# === Backend ===

# Start backend dev server
dev-be:
    bun scripts/dev.ts -b

# Run any command in backend directory
be *args:
    cd backend && {{args}}

# Run cargo commands in backend (e.g., `just cargo build`)
cargo *args:
    cd backend && cargo {{args}}

# === Agent ===

# Run agent in development mode (requires backend running)
dev-agent:
    cargo run --manifest-path agent/Cargo.toml

# Run agent once and exit (for testing single job)
dev-agent-once:
    cargo run --manifest-path agent/Cargo.toml -- --once

# Run agent in dev mode - direct shader+scene capture (bypasses job queue)
# Usage: just dev-agent-direct bsl-shaders mountain-noon,village-sunset
dev-agent-direct shader scenes:
    cargo run --manifest-path agent/Cargo.toml -- --dev-shader {{shader}} --dev-scenes {{scenes}}

# Check agent code
check-agent:
    cargo fmt --manifest-path agent/Cargo.toml -- --check
    cargo clippy --manifest-path agent/Cargo.toml -- --deny warnings

# Format agent code
format-agent:
    cargo fmt --manifest-path agent/Cargo.toml

# Test agent
test-agent:
    cargo nextest run --manifest-path agent/Cargo.toml

# === Mod Development ===

# Run Minecraft client (Fabric by default)
dev-mod platform="fabric":
    bun scripts/dev.ts -m -p {{platform}}

# Quick compile check for mod
check-mod:
    cd mod && ./gradlew :common:compileKotlin :common:compileJava --quiet

# Format mod code
format-mod:
    cd mod && ./gradlew spotlessApply ktlintFormat --quiet

# Check mod code formatting
format-check-mod:
    cd mod && ./gradlew spotlessCheck ktlintCheck --quiet

# Run mod tests
test-mod:
    cd mod && ./gradlew test --quiet

# Build mod production artifacts
build-mod:
    cd mod && ./gradlew build --quiet

# Integration test - verify client starts and mixins load
smoke platform="fabric":
    bun ./scripts/smoke.ts {{platform}}

# Run autonomous shader capture orchestration
orchestrate platform="fabric":
    cd mod && GLINT_AUTONOMOUS=true ./gradlew :{{platform}}:runClient

# Clean mod build
clean-mod:
    cd mod && ./gradlew clean

# Run any command in mod directory
mod *args:
    cd mod && {{args}}

# === Database ===

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
    cargo test --manifest-path backend/Cargo.toml export_bindings --quiet

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
