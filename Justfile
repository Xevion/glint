set positional-arguments := true

alias c := check
alias cf := check-fix
alias d := dev
alias t := test
alias f := format
alias l := lint
alias s3 := r2

default:
    @just --list

# Validate code: pre-flight + format + lint + test + security. Targets: backend,frontend,mod
check *targets:
    bunx tempo check {{targets}}

# Validate with auto-fix. Targets: backend,frontend,mod
check-fix *targets:
    bunx tempo check --fix {{targets}}

# Run tests. Targets: backend,frontend,mod. Flags: --e2e
test *targets:
    bunx tempo run test {{targets}}

# Lint code. Targets: backend,frontend,mod
lint *targets:
    bunx tempo lint {{targets}}

# Auto-format code. Targets: backend,frontend,mod
format *targets:
    bunx tempo fmt {{targets}}

# Dev server. Targets: frontend,backend,mod
dev *targets:
    bunx tempo dev {{targets}}

# Integration test - verify client starts and mixins load
smoke *flags:
    bunx tempo run smoke {{flags}}

# Run autonomous shader capture
orchestrate *flags:
    bunx tempo run orchestrate {{flags}}

# Docker operations. Usage: just docker [build|run] <web|capture>
docker *args:
    bunx tempo run docker {{args}}

# Query Minecraft source JAR
mcjar +args='':
    bunx tempo run mcjar {{args}}

# Manage dev services (PostgreSQL + MinIO). Commands: start (default), reset, rm
db *args:
    bunx tempo run db {{args}}

# Manage database migrations. Commands: run, reset, create <name>
migrate *args:
    bunx tempo run migrate {{args}}

# Regenerate TypeScript bindings from Rust types + GraphQL schema
bindings:
    bunx tempo run bindings

# Run aws s3 commands against the R2 bucket. R2/ expands to s3://<bucket>/
r2 *args:
    bunx tempo run r2 {{args}}

# Fix Loom remapping cache corruption (stale intermediary names in mixin annotations)
loom-fix:
    bunx tempo run loom-fix

# Install git pre-commit hooks
install-hooks:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "bunx tempo pre-commit" > .husky/pre-commit
    chmod +x .husky/pre-commit
    echo "Pre-commit hook installed"
