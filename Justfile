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
