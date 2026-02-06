/**
 * Run all project checks in parallel. Auto-fixes formatting when safe.
 *
 * Usage: bun scripts/check.ts [--fix|-f]
 */

import { c, elapsed, isStderrTTY } from "./lib/fmt";
import { run, runPiped, spawnCollect, raceInOrder, type CollectResult } from "./lib/proc";
import { existsSync, statSync, readdirSync, rmSync } from "fs";

const args = process.argv.slice(2);
let fix = false;

for (const arg of args) {
  if (arg === "-f" || arg === "--fix") {
    fix = true;
  } else {
    console.error(`Unknown flag: ${arg}`);
    process.exit(1);
  }
}

if (fix) {
  console.log(c("1;36", "→ Fixing..."));
  run(["bun", "run", "--cwd", "frontend", "format"]);
  run(["cargo", "fmt", "--manifest-path", "backend/Cargo.toml"]);
  runPiped(["./gradlew", "spotlessApply", "ktlintFormat", "--quiet"], { cwd: "mod" });
  console.log(c("1;36", "→ Verifying..."));
}

{
  const BINDINGS_DIR = "frontend/src/lib/bindings";

  let newestSrcMtime = 0;
  for (const file of new Bun.Glob("**/*.rs").scanSync("backend/src")) {
    const mt = statSync(`backend/src/${file}`).mtimeMs;
    if (mt > newestSrcMtime) newestSrcMtime = mt;
  }
  for (const f of ["backend/Cargo.toml", "backend/Cargo.lock"]) {
    if (existsSync(f)) {
      const mt = statSync(f).mtimeMs;
      if (mt > newestSrcMtime) newestSrcMtime = mt;
    }
  }

  let newestBindingMtime = 0;
  if (existsSync(BINDINGS_DIR)) {
    for (const file of new Bun.Glob("**/*").scanSync(BINDINGS_DIR)) {
      const mt = statSync(`${BINDINGS_DIR}/${file}`).mtimeMs;
      if (mt > newestBindingMtime) newestBindingMtime = mt;
    }
  }

  const stale = newestBindingMtime === 0 || newestSrcMtime > newestBindingMtime;
  if (stale) {
    const t = Date.now();
    process.stdout.write(
      c("1;36", "→ Regenerating TypeScript bindings (Rust sources changed)...") + "\n",
    );
    run(["cargo", "test", "--no-run", "--quiet"], { cwd: "backend" });
    rmSync(BINDINGS_DIR, { recursive: true, force: true });
    run(["cargo", "test", "export_bindings", "--quiet"], { cwd: "backend" });

    run(["bun", "scripts/bindings-barrel.ts"]);

    const count = readdirSync(BINDINGS_DIR).filter((f) => f.endsWith(".ts") && f !== "index.ts").length;
    process.stdout.write(c("32", "✓ bindings") + ` (${elapsed(t)}s, ${count} types)\n`);
  } else {
    process.stdout.write(c("2", "· bindings up-to-date, skipped") + "\n");
  }
}

interface Check {
  name: string;
  cmd: string[];
  cwd?: string;
  hint?: string;
  subsystem: "frontend" | "backend" | "mod" | "security";
}

const checks: Check[] = [
  // Frontend checks (3)
  {
    name: "frontend-check",
    subsystem: "frontend",
    cmd: ["bun", "run", "--cwd", "frontend", "check"],
  },
  {
    name: "frontend-lint",
    subsystem: "frontend",
    cmd: ["bun", "run", "--cwd", "frontend", "lint"],
  },
  {
    name: "frontend-format",
    subsystem: "frontend",
    cmd: ["bun", "run", "--cwd", "frontend", "format:check"],
    hint: "Run 'bun run --cwd frontend format' to fix formatting.",
  },
  // Backend checks (3)
  {
    name: "backend-format",
    subsystem: "backend",
    cmd: ["cargo", "fmt", "--manifest-path", "backend/Cargo.toml", "--", "--check"],
    hint: "Run 'cargo fmt --manifest-path backend/Cargo.toml' to fix formatting.",
  },
  {
    name: "backend-lint",
    subsystem: "backend",
    cmd: ["cargo", "clippy", "--manifest-path", "backend/Cargo.toml", "--", "--deny", "warnings"],
  },
  {
    name: "backend-test",
    subsystem: "backend",
    cmd: [
      "cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml",
      "-E", "not test(export_bindings)",
    ],
  },
  {
    name: "backend-sqlx",
    subsystem: "backend",
    cmd: ["cargo", "sqlx", "prepare", "--check"],
    cwd: "backend",
    hint: "Run 'cd backend && cargo sqlx prepare' to update query metadata.",
  },
  // Mod checks (4)
  {
    name: "mod-format",
    subsystem: "mod",
    cmd: ["./gradlew", "spotlessCheck", "ktlintCheck", "--quiet"],
    cwd: "mod",
    hint: "Run 'cd mod && ./gradlew spotlessApply ktlintFormat --quiet' to fix formatting.",
  },
  {
    name: "mod-lint",
    subsystem: "mod",
    cmd: ["./gradlew", "spotlessCheck", "ktlintCheck", "--quiet"],
    cwd: "mod",
  },
  {
    name: "mod-check",
    subsystem: "mod",
    cmd: ["./gradlew", ":common:compileKotlin", ":common:compileJava", "--quiet"],
    cwd: "mod",
  },
  {
    name: "mod-test",
    subsystem: "mod",
    cmd: ["./gradlew", "test", "--quiet"],
    cwd: "mod",
  },
  // Frontend unit tests
  {
    name: "frontend-test",
    subsystem: "frontend",
    cmd: ["bun", "run", "--cwd", "frontend", "test:unit"],
  },
  // Security audits (2)
  {
    name: "backend-audit",
    subsystem: "security",
    cmd: ["cargo", "audit"],
    cwd: "backend",
  },
  {
    name: "frontend-audit",
    subsystem: "security",
    cmd: ["bun", "audit", "--audit-level=moderate"],
    cwd: "frontend",
  },
];

const domains: Record<
  string,
  {
    peers: string[];
    format: () => ReturnType<typeof runPiped>;
    recheck: Check[];
  }
> = {
  "frontend-format": {
    peers: ["frontend-check", "frontend-lint", "frontend-test"],
    format: () => runPiped(["bun", "run", "--cwd", "frontend", "format"]),
    recheck: [
      {
        name: "frontend-format",
        subsystem: "frontend",
        cmd: ["bun", "run", "--cwd", "frontend", "format:check"],
      },
      {
        name: "frontend-check",
        subsystem: "frontend",
        cmd: ["bun", "run", "--cwd", "frontend", "check"],
      },
    ],
  },
  "backend-format": {
    peers: ["backend-lint", "backend-test"],
    format: () => runPiped(["cargo", "fmt", "--manifest-path", "backend/Cargo.toml"]),
    recheck: [
      {
        name: "backend-format",
        subsystem: "backend",
        cmd: ["cargo", "fmt", "--manifest-path", "backend/Cargo.toml", "--", "--check"],
      },
      {
        name: "backend-lint",
        subsystem: "backend",
        cmd: ["cargo", "clippy", "--manifest-path", "backend/Cargo.toml", "--", "--deny", "warnings"],
      },
    ],
  },
  "mod-format": {
    peers: ["mod-lint", "mod-check", "mod-test"],
    format: () => runPiped(["./gradlew", "spotlessApply", "ktlintFormat", "--quiet"], { cwd: "mod" }),
    recheck: [
      {
        name: "mod-format",
        subsystem: "mod",
        cmd: ["./gradlew", "spotlessCheck", "ktlintCheck", "--quiet"],
        cwd: "mod",
      },
      {
        name: "mod-check",
        subsystem: "mod",
        cmd: ["./gradlew", ":common:compileKotlin", ":common:compileJava", "--quiet"],
        cwd: "mod",
      },
    ],
  },
};

const start = Date.now();
const remaining = new Set(checks.map((ch) => ch.name));

const promises = checks.map(async (check) => ({
  ...check,
  ...(await spawnCollect(check.cmd, start, { cwd: check.cwd })),
}));

const interval = isStderrTTY
  ? setInterval(() => {
      process.stderr.write(`\r\x1b[K${elapsed(start)}s [${Array.from(remaining).join(", ")}]`);
    }, 100)
  : null;

const results: Record<string, Check & CollectResult> = {};

await raceInOrder(promises, checks, (r) => {
  results[r.name] = r;
  remaining.delete(r.name);
  if (isStderrTTY) process.stderr.write("\r\x1b[K");

  const subsystemLabel = c("2", `[${r.subsystem}]`);
  if (r.exitCode !== 0) {
    process.stdout.write(c("31", `✗ ${r.name}`) + ` ${subsystemLabel} (${r.elapsed}s)\n`);
    if (r.hint) {
      process.stdout.write(c("2", `  ${r.hint}`) + "\n");
    } else {
      if (r.stdout) process.stdout.write(r.stdout);
      if (r.stderr) process.stderr.write(r.stderr);
    }
  } else {
    process.stdout.write(c("32", `✓ ${r.name}`) + ` ${subsystemLabel} (${r.elapsed}s)\n`);
  }
});

if (interval) clearInterval(interval);
if (isStderrTTY) process.stderr.write("\r\x1b[K");

const autoFixedDomains = new Set<string>();

for (const [fmtName, domain] of Object.entries(domains)) {
  const fmtResult = results[fmtName];
  if (!fmtResult || fmtResult.exitCode === 0) continue;
  if (!domain.peers.every((p) => results[p]?.exitCode === 0)) continue;

  process.stdout.write(
    "\n" +
      c("1;36", `→ Auto-formatting ${fmtName} (peers passed, only formatting failed)...`) +
      "\n",
  );
  const fmtOut = domain.format();
  if (fmtOut.exitCode !== 0) {
    process.stdout.write(c("31", `  ✗ ${fmtName} formatter failed`) + "\n");
    if (fmtOut.stdout) process.stdout.write(fmtOut.stdout);
    if (fmtOut.stderr) process.stderr.write(fmtOut.stderr);
    continue;
  }

  const recheckStart = Date.now();
  const recheckPromises = domain.recheck.map(async (ch) => ({
    ...ch,
    ...(await spawnCollect(ch.cmd, recheckStart, { cwd: ch.cwd })),
  }));

  let recheckFailed = false;
  await raceInOrder(recheckPromises, domain.recheck, (r) => {
    if (r.exitCode !== 0) {
      recheckFailed = true;
      process.stdout.write(c("31", `  ✗ ${r.name}`) + ` (${r.elapsed}s)\n`);
      if (r.stdout) process.stdout.write(r.stdout);
      if (r.stderr) process.stderr.write(r.stderr);
    } else {
      process.stdout.write(c("32", `  ✓ ${r.name}`) + ` (${r.elapsed}s)\n`);
    }
  });

  if (!recheckFailed) {
    process.stdout.write(c("32", `  ✓ ${fmtName} auto-fix succeeded`) + "\n");
    autoFixedDomains.add(fmtName);
  } else {
    process.stdout.write(c("31", `  ✗ ${fmtName} auto-fix failed sanity check`) + "\n");
  }
}

const finalFailed = Object.entries(results).some(
  ([name, r]) => r.exitCode !== 0 && !autoFixedDomains.has(name),
);

if (autoFixedDomains.size > 0 && !finalFailed) {
  process.stdout.write(
    "\n" + c("1;32", "✓ All checks passed (formatting was auto-fixed)") + "\n",
  );
}

process.exit(finalFailed ? 1 : 0);
