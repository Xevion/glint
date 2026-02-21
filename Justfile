set positional-arguments := true

alias c := check
alias d := dev
alias t := test
alias f := format
alias l := lint
alias dm := dev-mod
alias sm := smoke
alias lf := loom-fix

default:
    @just --list

# Validate code: pre-flight + format + lint + test + security. Targets: backend,frontend,mod
check *targets:
    bun scripts/check.ts {{targets}}

# Run tests. Targets: backend,frontend,mod,e2e
test *targets:
    bun scripts/test.ts {{targets}}

# Lint code. Targets: backend,frontend,mod
lint *targets:
    bun scripts/lint.ts {{targets}}

# Auto-format code. Targets: backend,frontend,mod
format *targets:
    bun scripts/format.ts {{targets}}

# Dev server: -f(rontend) -b(ackend) -m(od) -W(no-watch) -r(elease) -p(latform) -- <args>
dev *flags:
    bun scripts/dev.ts {{flags}}

# Run Minecraft client (Fabric by default)
dev-mod platform="fabric":
    bun scripts/dev.ts -m -p {{platform}}

# Run autonomous shader capture
orchestrate *flags:
    bun scripts/orchestrate.ts {{flags}}

# Integration test - verify client starts and mixins load
smoke platform="fabric":
    bun ./scripts/smoke.ts {{platform}}

# Manage dev services (PostgreSQL + MinIO). Commands: start (default), reset, rm
db cmd="start":
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{cmd}}" in
        start)
            docker compose up -d
            bun scripts/db-init.ts
            ;;
        reset)
            docker compose up -d
            docker compose exec postgres psql -U glint -d postgres -c "DROP DATABASE IF EXISTS glint"
            docker compose exec postgres psql -U glint -d postgres -c "CREATE DATABASE glint"
            bun scripts/db-init.ts
            ;;
        rm)
            docker compose down
            ;;
        *)
            echo "Unknown command: {{cmd}}" >&2
            exit 1
            ;;
    esac

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

# Docker operations. Usage: just docker [build|run] <web|capture>
docker *args:
    bun scripts/docker.ts {{args}}

# Regenerate TypeScript bindings from Rust types + GraphQL schema
bindings:
    cd backend && cargo test export_bindings --quiet
    bun scripts/bindings-barrel.ts
    cd backend && cargo test --test graphql_schema --quiet
    cd frontend && bunx gql-tada generate output

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

alias s3 := r2

# Query Minecraft source JAR
# Usage:
#   just mcjar list net/minecraft/client/renderer/
#   just mcjar cat net/minecraft/client/Minecraft.java
#   just mcjar grep shouldEntityAppearGlowing net/minecraft/client/Minecraft.java
#   just mcjar grep-all startUseItem 'net/minecraft/client/*.java'
#   just mcjar asset rendertype_lines.vsh
#   just mcjar asset-list shaders/
mcjar +args='':
    #!/usr/bin/env bash
    set -euo pipefail
    exec bun ./scripts/mcjar.ts "$@"

# Run bun commands in frontend (e.g., `just bun run test`)
bun *args:
    cd frontend && bun {{args}}

# Run commands in mod directory (e.g., `just mod ./gradlew clean`)
mod *args:
    cd mod && {{args}}

# Fix Loom remapping cache corruption (stale intermediary names in mixin annotations)
loom-fix:
    rm -rf mod/.gradle/loom-cache/remapped_mods
    cd mod && ./gradlew --stop
    cd mod && ./gradlew :fabric:configureClientLaunch --refresh-dependencies
    @echo "✓ Loom remapped_mods cache rebuilt"

# Install git pre-commit hooks
install-hooks:
    #!/usr/bin/env bash
    set -euo pipefail
    chmod +x scripts/pre-commit.ts
    echo "bun scripts/pre-commit.ts" > .husky/pre-commit
    chmod +x .husky/pre-commit
    echo "✓ Pre-commit hook installed"
