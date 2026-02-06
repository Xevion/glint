/**
 * Dev server orchestrator for Glint's three subsystems.
 *
 * Usage: bun scripts/dev.ts [flags] [-- passthrough-args]
 *
 * Flags:
 *   -f, --frontend-only   Frontend only (Vite dev server)
 *   -b, --backend-only    Backend only (watch + seamless reload)
 *   -m, --mod-only        Minecraft client only (Gradle)
 *   -W, --no-watch        Build once + run (no watch)
 *   -r, --release         Use release profile (Rust)
 *   -p, --platform        Mod platform: fabric (default) or neoforge
 *   -I, --no-interrupt    Don't kill compiler on new changes; wait, then rebuild
 *   -V, --verbose-build   Stream compilation output inline (default: buffered)
 *   -- <args>             Passthrough args to backend binary
 */

import { existsSync } from "fs";
import { parseFlags, c } from "./lib/fmt";
import { run, ProcessGroup } from "./lib/proc";
import { BackendWatcher } from "./lib/watch";

const { flags, passthrough } = parseFlags(
  process.argv.slice(2),
  {
    "frontend-only": "bool",
    "backend-only": "bool",
    "mod-only": "bool",
    "no-watch": "bool",
    release: "bool",
    platform: "string",
    "no-interrupt": "bool",
    "verbose-build": "bool",
  } as const,
  {
    f: "frontend-only",
    b: "backend-only",
    m: "mod-only",
    W: "no-watch",
    r: "release",
    p: "platform",
    I: "no-interrupt",
    V: "verbose-build",
  },
  {
    "frontend-only": false,
    "backend-only": false,
    "mod-only": false,
    "no-watch": false,
    release: false,
    platform: "fabric",
    "no-interrupt": false,
    "verbose-build": false,
  },
);

const frontendOnly = flags["frontend-only"];
const backendOnly = flags["backend-only"];
const modOnly = flags["mod-only"];
const noWatch = flags["no-watch"];
const release = flags.release;
const platform = flags.platform as string;

// Validate platform
if (platform !== "fabric" && platform !== "neoforge") {
  console.error(`Invalid platform: ${platform} (must be 'fabric' or 'neoforge')`);
  process.exit(1);
}

// Determine what to run based on flags
// Default: frontend + backend
// If any -f/-b/-m specified: run only those
const anySubsystemSpecified = frontendOnly || backendOnly || modOnly;

const runFrontend = anySubsystemSpecified ? frontendOnly : true;
const runBackend = anySubsystemSpecified ? backendOnly : true;
const runMod = modOnly; // Only runs if explicitly requested

const profile = release ? "release" : "debug";
const group = new ProcessGroup();

// Frontend: Vite dev server
if (runFrontend) {
  console.log(c("1;36", "→ Starting frontend dev server..."));
  group.spawn(["bun", "run", "--cwd", "frontend", "dev"]);
}

// Backend: seamless watch+reload or build-once
if (runBackend) {
  const backendArgs = passthrough;
  const bin = `backend/target/${profile}/glint`;

  if (noWatch) {
    console.log(c("1;36", `→ Building backend (${release ? "release" : "debug"})...`));
    const cargoArgs = ["cargo", "build", "--manifest-path", "backend/Cargo.toml"];
    if (release) cargoArgs.push("--release");
    run(cargoArgs);

    if (!existsSync(bin)) {
      console.error(`Binary not found: ${bin}`);
      console.error(`Build failed or binary name mismatch.`);
      await group.killAll();
      process.exit(1);
    }

    console.log(c("1;36", `→ Running ${bin} (no watch)`));
    group.spawn([bin, ...backendArgs]);
  } else {
    console.log(c("1;36", "→ Starting backend dev server (watch mode)..."));
    const watcher = new BackendWatcher({
      manifestPath: "backend/Cargo.toml",
      binPath: bin,
      release,
      args: backendArgs,
      interrupt: !flags["no-interrupt"],
      verboseBuild: flags["verbose-build"],
    });
    group.onCleanup(() => watcher.killSync());
    watcher.start();
  }
}

// Mod: Minecraft client via Gradle
if (runMod) {
  console.log(c("1;36", `→ Starting Minecraft client (${platform})...`));
  group.spawn(["./gradlew", `--console=plain`, `:${platform}:runClient`], { cwd: "mod" });
}

const code = await group.waitForFirst();
process.exit(code);
