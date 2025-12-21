# Glint - Shader Preview Catalog

default:
    @just --list

# Run both frontend and backend dev servers
dev:
    @just dev-fe & just dev-be & wait

# Start frontend dev server
dev-fe:
    bun run --cwd frontend dev

# Start backend dev server
dev-be:
    cargo run --manifest-path backend/Cargo.toml

# Validate all code (typecheck, clippy)
check:
    bun run --cwd frontend check
    cargo fmt --manifest-path backend/Cargo.toml -- --check
    cargo clippy --manifest-path backend/Cargo.toml -- --deny warnings

# Auto-format all code
format:
    bun run --cwd frontend format
    cargo fmt --manifest-path backend/Cargo.toml

# Lint all code
lint:
    bun run --cwd frontend lint
    cargo clippy --manifest-path backend/Cargo.toml -- --deny warnings

# Build everything for production
build:
    bun run --cwd frontend build
    cargo build --manifest-path backend/Cargo.toml --release

# Run all tests
test:
    bun run --cwd frontend test
    cargo nextest run --manifest-path backend/Cargo.toml

# Run bun commands in frontend (e.g., `just bun run test`)
bun *args:
    cd frontend && bun {{args}}

# Run cargo commands in backend (e.g., `just cargo build`)
cargo *args:
    cd backend && cargo {{args}}

# Run any command in frontend
fe *args:
    cd frontend && {{args}}

# Run any command in backend
be *args:
    cd backend && {{args}}

# Run any command in mod
mod *args:
	cd mod && {{args}}

# --- Mod Development ---

# Quick compile check for mod
check-mod:
	cd mod && ./gradlew :common:compileKotlin :common:compileJava --quiet

# Format mod code
format-mod:
	cd mod && ./gradlew spotlessApply --quiet

# Check mod code formatting
format-check-mod:
	cd mod && ./gradlew spotlessCheck --quiet

# Run mod tests
test-mod:
	cd mod && ./gradlew test --quiet

# Build mod production artifacts
build-mod:
	cd mod && ./gradlew build --quiet

# Run Minecraft client (Fabric by default)
dev-mod platform="fabric":
	cd mod && ./gradlew :{{platform}}:runClient

# Clean mod build
clean-mod:
	cd mod && ./gradlew clean

# Integration test - verify client starts and mixins load
smoke platform="fabric":
	bun ./scripts/smoke.ts {{platform}}

# Run autonomous shader capture orchestration
orchestrate platform="fabric":
	cd mod && GLINT_AUTONOMOUS=true ./gradlew :{{platform}}:runClient

# Query Minecraft source JAR
# Usage:
#   just mcjar list net/minecraft/client/renderer/        # List classes in package
#   just mcjar cat net/minecraft/client/Minecraft.java    # Read entire class
#   just mcjar grep shouldEntityAppearGlowing net/minecraft/client/Minecraft.java  # Search in class
#   just mcjar grep-all startUseItem 'net/minecraft/client/*.java'  # Search multiple files
#   just mcjar asset rendertype_lines.vsh  # Read shader/asset file
#   just mcjar asset-list shaders/  # List asset files
mcjar cmd *args:
	bun ./scripts/mcjar.ts {{cmd}} {{args}}
