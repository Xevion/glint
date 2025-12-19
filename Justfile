# Glint - Shader Preview Catalog

default:
    @just --list

# Run both frontend and backend dev servers
dev:
    @just dev-frontend & just dev-backend & wait

# Start frontend dev server
dev-frontend:
    bun run --cwd frontend dev

# Start backend dev server
dev-backend:
    cargo run --manifest-path backend/Cargo.toml

# Validate all code (format, lint, typecheck)
check:
    cargo fmt --manifest-path backend/Cargo.toml -- --check
    cargo clippy --manifest-path backend/Cargo.toml -- --deny warnings
    bun run --cwd frontend check

# Auto-format all code
format:
    cargo fmt --manifest-path backend/Cargo.toml
    bun run --cwd frontend format

# Build everything for production
build:
    cargo build --manifest-path backend/Cargo.toml --release
    bun run --cwd frontend build

# Run backend tests
test:
    cargo nextest run --manifest-path backend/Cargo.toml

# Run any bun command in frontend
bun *args:
    cd frontend && bun {{args}}

# Run any cargo command in backend
cargo *args:
    cd backend && cargo {{args}}

# Shorthand for frontend commands
fe *args:
    cd frontend && {{args}}

# Shorthand for backend commands
be *args:
    cd backend && {{args}}
