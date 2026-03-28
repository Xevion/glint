import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defineConfig, presets } from "@xevion/tempo";
import { newestMtime } from "@xevion/tempo/preflight";
import { resolveTargets } from "@xevion/tempo/targets";

/** Parse a .env file into a key-value record (ignores comments and blank lines) */
function loadEnvFile(path: string): Record<string, string> {
  const env: Record<string, string> = {};
  if (!existsSync(path)) return env;
  for (const line of readFileSync(path, "utf-8").split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const eq = trimmed.indexOf("=");
    if (eq === -1) continue;
    const key = trimmed.slice(0, eq).trim();
    let val = trimmed.slice(eq + 1).trim();
    // Strip surrounding quotes
    if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
      val = val.slice(1, -1);
    }
    env[key] = val;
  }
  return env;
}

const BINDINGS_DIR = "frontend/src/lib/bindings";

/** Generate the barrel (index.ts) for TypeScript bindings */
function generateBarrel(): void {
  const types = readdirSync(BINDINGS_DIR)
    .filter((f) => f.endsWith(".ts") && f !== "index.ts")
    .map((f) => f.replace(/\.ts$/, ""))
    .sort();

  const barrelContent =
    "// Auto-generated barrel file — do not edit manually.\n" +
    "// Regenerate with: cd backend && cargo test export_bindings\n" +
    types.map((t) => `export type { ${t} } from "./${t}";`).join("\n") +
    "\n";

  const barrelPath = join(BINDINGS_DIR, "index.ts");
  const existing = existsSync(barrelPath) ? readFileSync(barrelPath, "utf-8") : null;

  if (existing !== barrelContent) {
    writeFileSync(barrelPath, barrelContent);
  }
}

const DATABASE_URL = "postgresql://glint:glint@localhost:59490/glint";

/** Write DATABASE_URL to backend/.env after docker compose up */
async function ensureDatabaseUrl(): Promise<void> {
  const envFile = "backend/.env";
  let content: string;
  try {
    content = await readFile(envFile, "utf8");
    const regex = /^DATABASE_URL=.*$/m;
    if (regex.test(content)) {
      content = content.replace(regex, `DATABASE_URL=${DATABASE_URL}`);
    } else {
      content = content.trimEnd() + `\nDATABASE_URL=${DATABASE_URL}\n`;
    }
  } catch {
    content = `DATABASE_URL=${DATABASE_URL}\n`;
  }
  await writeFile(envFile, content);
  console.log(`Updated ${envFile}`);
}

// Shared: newest mtime across backend Rust sources + Cargo files
function rustSrcMtime(): number {
  return Math.max(
    newestMtime("backend/src", "**/*.rs"),
    ...["backend/Cargo.toml", "backend/Cargo.lock"]
      .filter(existsSync)
      .map((f) => statSync(f).mtimeMs),
  );
}

const rustPreset = presets.rust({
  manifestPath: "backend/Cargo.toml",
  testFilter: "not test(export_bindings) and not test(export_graphql_sdl)",
  bin: "glint",
});

const gradlePreset = presets.gradle({
  cwd: "mod",
  subprojects: ["common"],
});

// Advisory IDs to ignore in bun audit (transitive, no fix available)
const IGNORED_ADVISORIES = [
  "GHSA-2g4f-4pwh-qvx6", // ajv ReDoS — transitive via eslint
];

const bunAuditCmd = [
  "bun", "audit", "--audit-level=moderate",
  ...IGNORED_ADVISORIES.map((id) => `--ignore=${id}`),
].join(" ");

export default defineConfig({
  subsystems: {
    frontend: {
      aliases: ["f", "front", "web"],
      cwd: "frontend",
      commands: {
        "format-check": "bunx biome format .",
        "format-apply": "bunx biome format --write .",
        lint: "eslint --cache .",
        "type-check": "bun run check",
        test: "bunx vitest run",
        build: "bunx --bun vite build",
      },
      autoFix: {
        "format-check": "format-apply",
      },
    },
    backend: {
      ...rustPreset,
      aliases: ["b", "back", "rust"],
      commands: {
        ...rustPreset.commands,
        lint: {
          cmd: rustPreset.commands.lint as string,
          env: { SQLX_OFFLINE: "true" },
        },
        test: {
          cmd: rustPreset.commands.test as string,
          env: { SQLX_OFFLINE: "true" },
        },
        build: {
          cmd: rustPreset.commands.build as string,
          env: { SQLX_OFFLINE: "true" },
        },
      },
    },
    mod: {
      ...gradlePreset,
      aliases: ["m", "minecraft"],
    },
    security: {
      alwaysRun: true,
      aliases: ["sec", "audit"],
      commands: {
        "cargo-audit": {
          cmd: "cargo audit -f backend/Cargo.lock",
          requires: ["cargo-audit"],
          hint: "Install cargo-audit: cargo install cargo-audit",
        },
        "bun-audit": {
          cmd: bunAuditCmd,
          cwd: "frontend",
        },
      },
    },
  },
  preflights: [
    // Ensure frontend node_modules exist
    (ctx) => {
      if (!existsSync("frontend/node_modules")) {
        ctx.fail("frontend/node_modules not found -- run `bun install --cwd frontend` first");
      }
    },

    // TS bindings: Rust types → frontend TypeScript (complex: temp dir + diffing + barrel)
    async (ctx) => {
      const srcMtime = rustSrcMtime();
      const artifactMtime = newestMtime(BINDINGS_DIR, "**/*");

      if (artifactMtime >= srcMtime) return;

      ctx.logger.info("Regenerating TypeScript bindings (Rust sources changed)...");
      const tmpDir = mkdtempSync(join(tmpdir(), "glint-bindings-"));

      try {
        const { spawnSync } = await import("node:child_process");

        const offlineEnv = { ...process.env, SQLX_OFFLINE: "true" };

        // Build test binary first
        const build = spawnSync("cargo", ["test", "--lib", "--no-run", "--quiet"], {
          cwd: "backend",
          stdio: ["ignore", "pipe", "pipe"],
          env: offlineEnv,
        });
        if (build.status !== 0) {
          if (build.stderr?.length) process.stderr.write(build.stderr);
          ctx.fail("Failed to build bindings test binary");
          return;
        }

        // Export bindings to temp dir
        const exp = spawnSync("cargo", ["test", "--lib", "export_bindings", "--quiet"], {
          cwd: "backend",
          stdio: ["ignore", "pipe", "pipe"],
          env: { ...offlineEnv, TS_RS_EXPORT_DIR: tmpDir },
        });
        if (exp.status !== 0) {
          if (exp.stdout?.length) process.stdout.write(exp.stdout);
          if (exp.stderr?.length) process.stderr.write(exp.stderr);
          ctx.fail("Failed to export bindings");
          return;
        }

        if (!existsSync(BINDINGS_DIR)) {
          mkdirSync(BINDINGS_DIR, { recursive: true });
        }

        const newFiles = new Set(readdirSync(tmpDir).filter((f) => f.endsWith(".ts")));
        const oldFiles = new Set(
          readdirSync(BINDINGS_DIR).filter((f) => f.endsWith(".ts") && f !== "index.ts"),
        );

        let changed = 0;
        for (const file of newFiles) {
          const newContent = readFileSync(join(tmpDir, file));
          const oldPath = join(BINDINGS_DIR, file);
          if (existsSync(oldPath)) {
            const oldContent = readFileSync(oldPath);
            if (Buffer.compare(newContent, oldContent) === 0) continue;
          }
          writeFileSync(oldPath, newContent);
          changed++;
        }

        for (const file of oldFiles) {
          if (!newFiles.has(file)) {
            rmSync(join(BINDINGS_DIR, file));
            changed++;
          }
        }

        // Regenerate barrel file
        try {
          generateBarrel();
        } catch {
          ctx.logger.warn("Barrel generation failed (non-fatal)");
        }

        ctx.logger.info(`Bindings: ${newFiles.size} types, ${changed} changed`);
      } finally {
        rmSync(tmpDir, { recursive: true, force: true });
      }
    },

    // JSON schemas: Rust types → schemas/ (mod contract testing)
    {
      label: "schemas",
      sources: { dir: "backend/src", pattern: "**/*.rs" },
      artifacts: { dir: "schemas", pattern: "*.json" },
      regenerate: "SQLX_OFFLINE=true cargo test export_all_schemas --quiet --manifest-path backend/Cargo.toml",
      reason: "Rust sources changed",
    },

    // SQLx query metadata: Rust + migrations → backend/.sqlx/
    async (ctx) => {
      const srcMtime = Math.max(rustSrcMtime(), newestMtime("backend/migrations", "*.sql"));
      const artifactMtime = newestMtime("backend/.sqlx", "*.json");

      if (artifactMtime >= srcMtime) return;

      ctx.logger.info("Regenerating SQLx metadata (sources changed)...");
      const { spawnSync } = await import("node:child_process");
      const result = spawnSync("cargo", ["sqlx", "prepare"], {
        cwd: "backend",
        stdio: ["ignore", "pipe", "pipe"],
      });
      if (result.status !== 0) {
        ctx.logger.warn("sqlx prepare failed (is the database running?)");
        return;
      }
      const count = existsSync("backend/.sqlx")
        ? readdirSync("backend/.sqlx").filter((f) => f.endsWith(".json")).length
        : 0;
      ctx.logger.info(`SQLx: ${count} queries`);
    },

    // GraphQL schema: Rust → frontend/src/lib/graphql/schema.graphql
    async (ctx) => {
      const GQL_DIR = "frontend/src/lib/graphql";
      const schemaPath = join(GQL_DIR, "schema.graphql");
      const srcMtime = rustSrcMtime();
      const artifactMtime = newestMtime(GQL_DIR, "schema.graphql");

      if (artifactMtime >= srcMtime) return;

      ctx.logger.info("Regenerating GraphQL schema (Rust sources changed)...");
      const existing = existsSync(schemaPath) ? readFileSync(schemaPath) : null;

      const { spawnSync } = await import("node:child_process");
      const result = spawnSync("cargo", ["test", "--test", "graphql_schema", "--quiet"], {
        cwd: "backend",
        stdio: ["ignore", "pipe", "pipe"],
        env: { ...process.env, SQLX_OFFLINE: "true" },
      });
      if (result.status !== 0) {
        if (result.stdout?.length) process.stdout.write(result.stdout);
        if (result.stderr?.length) process.stderr.write(result.stderr);
        ctx.fail("GraphQL schema export failed");
        return;
      }

      const updated = readFileSync(schemaPath);
      const changed = !existing || Buffer.compare(existing, updated) !== 0;
      if (changed) ctx.logger.info("GraphQL schema: updated");
    },

    // gql-tada: schema.graphql → graphql-env.d.ts
    async (ctx) => {
      const GQL_DIR = "frontend/src/lib/graphql";
      const schemaMtime = newestMtime(GQL_DIR, "schema.graphql");
      const envMtime = newestMtime(GQL_DIR, "graphql-env.d.ts");

      if (envMtime >= schemaMtime) return;

      ctx.logger.info("Regenerating gql-tada output (schema changed)...");
      const { spawnSync } = await import("node:child_process");
      const result = spawnSync("bunx", ["gql-tada", "generate", "output"], {
        cwd: "frontend",
        stdio: ["ignore", "pipe", "pipe"],
      });
      if (result.status !== 0) {
        if (result.stdout?.length) process.stdout.write(result.stdout);
        if (result.stderr?.length) process.stderr.write(result.stderr);
        ctx.fail("gql-tada generation failed");
        return;
      }
      ctx.logger.info("gql-tada: updated");
    },
  ],
  check: {
    autoFixStrategy: "fix-first",
    exclude: [
      "frontend:build",       // don't build frontend during checks
      "backend:build",        // don't build backend during checks
    ],
  },
  hooks: {
    "before:dev": (ctx) => {
      if (ctx.targets.has("frontend") && !existsSync("frontend/node_modules")) {
        ctx.fail("frontend/node_modules not found -- run `bun install --cwd frontend` first");
      }
      if (ctx.targets.has("backend") && !existsSync("backend/.env")) {
        ctx.logger.warn("backend/.env not found -- copy backend/.env.example if needed");
      }
    },
  },
  dev: {
    exitBehavior: "first-exits",
    processes: {
      frontend: {
        type: "unmanaged",
        cmd: ["bun", "run", "dev"],
        cwd: "frontend",
      },
      backend: {
        type: "managed",
        watch: {
          dirs: ["backend/src", "backend/migrations"],
          exts: [".rs", ".sql"],
          extraPaths: ["backend/Cargo.toml", "backend/Cargo.lock"],
          debounce: 300,
        },
        build: {
          cmd: ["cargo", "build", "--manifest-path", "backend/Cargo.toml"],
        },
        run: {
          cmd: ["./backend/target/debug/glint"],
          passthrough: true,
        },
        interrupt: true,
      },
      mod: {
        type: "unmanaged",
        cmd: ["./gradlew", "--console=plain", ":fabric:runClient"],
        cwd: "mod",
      },
    },
  },
  custom: {
    smoke: "scripts/smoke.ts",
    orchestrate: "scripts/orchestrate.ts",
    docker: "scripts/docker.ts",
    mcjar: {
      description: "Query Minecraft source files from decompiled JAR",
      run: async (ctx) => {
        ctx.run(`bun scripts/mcjar.ts ${ctx.args.join(" ")}`);
        return 0;
      },
    },
    test: {
      description: "Run tests across subsystems in parallel",
      flags: {
        e2e: { type: Boolean, description: "Run Playwright E2E tests instead of unit tests for frontend" },
      },
      run: async (ctx) => {
        const targets = resolveTargets(ctx.args, ctx.config!.subsystems);
        const all = targets.subsystems.size === Object.keys(ctx.config!.subsystems).length;
        // When no targets specified, run all unit tests (exclude security — it has no tests)
        const subs = all
          ? new Set(["frontend", "backend", "mod"])
          : targets.subsystems;

        if (subs.has("frontend")) {
          if (ctx.flags.e2e) {
            ctx.group.spawn("bun run test:e2e", { cwd: "frontend" });
          } else {
            ctx.group.spawn("bunx vitest run", { cwd: "frontend" });
          }
        }
        if (subs.has("backend")) {
          ctx.group.spawn(["cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml"], {
            env: { SQLX_OFFLINE: "true" },
          });
        }
        if (subs.has("mod")) {
          ctx.group.spawn(["./gradlew", "test", "--quiet"], { cwd: "mod" });
        }

        const code = await ctx.group.waitForAll();
        return code;
      },
    },
    db: {
      description: "Manage dev services (PostgreSQL + MinIO + imgproxy)",
      run: async (ctx) => {
        const subcmd = ctx.args[0] || "start";
        switch (subcmd) {
          case "start":
            ctx.run("docker compose up -d");
            await ensureDatabaseUrl();
            break;
          case "reset":
            ctx.run("docker compose up -d");
            ctx.run(["docker", "compose", "exec", "postgres", "psql", "-U", "glint", "-d", "postgres", "-c", "DROP DATABASE IF EXISTS glint"]);
            ctx.run(["docker", "compose", "exec", "postgres", "psql", "-U", "glint", "-d", "postgres", "-c", "CREATE DATABASE glint"]);
            await ensureDatabaseUrl();
            break;
          case "rm":
            ctx.run("docker compose down");
            break;
          default:
            console.error(`Unknown db command: "${subcmd}"\nValid: start, reset, rm`);
            return 1;
        }
        return 0;
      },
    },
    migrate: {
      description: "Manage database migrations (SQLx)",
      run: async (ctx) => {
        const subcmd = ctx.args[0];
        switch (subcmd) {
          case "run": {
            ctx.run("cargo sqlx migrate run", { cwd: "backend" });
            const result = ctx.runPiped("bash -c 'source .env 2>/dev/null && psql \"$DATABASE_URL\" -f views.sql -q'", { cwd: "backend" });
            if (result.exitCode !== 0) {
              console.warn("⚠ psql not available or failed; views will be applied on next app startup");
            }
            break;
          }
          case "reset":
            ctx.run("cargo sqlx database reset -y", { cwd: "backend" });
            break;
          case "create": {
            const name = ctx.args[1];
            if (!name) {
              console.error("Usage: tempo run migrate create <name>");
              return 1;
            }
            ctx.run(`cargo sqlx migrate add ${name}`, { cwd: "backend" });
            break;
          }
          default:
            console.error(`Unknown migrate command: "${subcmd || "(none)"}"\nValid: run, reset, create <name>`);
            return 1;
        }
        return 0;
      },
    },
    bindings: {
      description: "Force-regenerate TypeScript bindings + GraphQL schema",
      run: async (ctx) => {
        const sqlxEnv = { SQLX_OFFLINE: "true" };
        ctx.run("cargo test --manifest-path backend/Cargo.toml export_bindings --quiet", { env: sqlxEnv });
        generateBarrel();
        ctx.run("cargo test --manifest-path backend/Cargo.toml --test graphql_schema --quiet", { env: sqlxEnv });
        ctx.run("bunx --cwd frontend gql-tada generate output");
        return 0;
      },
    },
    r2: {
      description: "Run aws s3 commands against the R2 bucket (R2/ expands to s3://<bucket>/)",
      run: async (ctx) => {
        const env = loadEnvFile("backend/.env");
        const accessKey = env.GLINT_R2_ACCESS_KEY_ID;
        const secretKey = env.GLINT_R2_SECRET_ACCESS_KEY;
        const accountId = env.GLINT_R2_ACCOUNT_ID;
        const bucketName = env.GLINT_R2_BUCKET;

        if (!accessKey || !secretKey || !accountId || !bucketName) {
          console.error("Missing R2 config in backend/.env (need GLINT_R2_ACCESS_KEY_ID, GLINT_R2_SECRET_ACCESS_KEY, GLINT_R2_ACCOUNT_ID, GLINT_R2_BUCKET)");
          return 1;
        }

        const bucket = `s3://${bucketName}`;
        const expandedArgs = ctx.args.map((arg) => {
          if (arg === "R2") return bucket;
          if (arg.startsWith("R2/")) return `${bucket}/${arg.slice(3)}`;
          return arg;
        });

        ctx.run(["aws", "s3", ...expandedArgs], {
          env: {
            AWS_ACCESS_KEY_ID: accessKey,
            AWS_SECRET_ACCESS_KEY: secretKey,
            AWS_ENDPOINT_URL: `https://${accountId}.r2.cloudflarestorage.com`,
          },
        });
        return 0;
      },
    },
    "loom-fix": {
      description: "Fix Loom remapping cache corruption (stale intermediary names)",
      run: async (ctx) => {
        ctx.run("rm -rf mod/.gradle/loom-cache/remapped_mods");
        ctx.run("./gradlew --stop", { cwd: "mod" });
        ctx.run("./gradlew :fabric:configureClientLaunch --refresh-dependencies", { cwd: "mod" });
        console.log("✓ Loom remapped_mods cache rebuilt");
        return 0;
      },
    },
  },
});
