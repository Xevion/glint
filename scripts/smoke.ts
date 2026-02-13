#!/usr/bin/env bun

/**
 * Minecraft smoke test — launches the game and verifies it starts.
 *
 * Spawns the Minecraft client via Gradle, monitors output for success
 * markers (OpenGL, mods loaded) and fatal errors (mixin crashes, build
 * failures), and exits with appropriate codes.
 *
 * Usage: bun scripts/smoke.ts [flags]
 *
 * Flags:
 *   -p, --platform    Mod platform: fabric (default) or neoforge
 *   -v, --verbose     Show full Minecraft output (default: env VERBOSE=1)
 *   -h, --help        Show this help message and exit
 *
 * Exit codes:
 *   0 — Success (game started, mixins loaded)
 *   1 — Build failed
 *   2 — Fatal mixin error
 *   3 — Timeout
 *   4 — Process died unexpectedly
 *   5 — Early crash (ClassNotFoundException, etc.)
 */

import { existsSync } from "fs";
import * as fs from "fs/promises";
import { randomUUID } from "crypto";
import { parseFlags, c } from "./lib/fmt";
import {
  GAME_STARTED_MARKERS,
  BUILD_ERROR_PATTERNS,
  FATAL_MIXIN_ERRORS,
  EARLY_CRASH_PATTERNS,
  isProcessAlive,
  sleep,
  extractErrorContext,
  ensureQuietOptions,
  monitorStream,
  spawnMinecraft,
  registerSignalHandlers,
  printFailureTail,
  type StreamBuffers,
  type PatternAction,
} from "./lib/minecraft";

const { flags } = parseFlags(
  process.argv.slice(2),
  {
    platform: "string",
    verbose: "bool",
    help: "bool",
  } as const,
  { p: "platform", v: "verbose", h: "help" },
  { platform: "fabric", verbose: false, help: false },
);

if (flags.help) {
  console.log(`Usage: bun scripts/smoke.ts [flags]

Launches Minecraft and verifies the game starts successfully.

Flags:
  -p, --platform <name>   Mod platform: fabric (default) or neoforge
  -v, --verbose            Show full Minecraft output (also: VERBOSE=1 env)
  -h, --help               Show this help message and exit

Exit codes:
  0  Success (game started, mixins loaded)
  1  Build failed
  2  Fatal mixin error
  3  Timeout
  4  Process died unexpectedly
  5  Early crash`);
  process.exit(0);
}

const platform = flags.platform as string;
const verbose = flags.verbose || !!process.env.VERBOSE;

if (platform !== "fabric" && platform !== "neoforge") {
  console.error(
    c("31", `Invalid platform: ${platform} (must be 'fabric' or 'neoforge')`),
  );
  process.exit(1);
}

const SESSION_ID = randomUUID();
const SESSION_FILE = `/tmp/glint-smoke-${SESSION_ID}.session`;

const ASSETS_PATH = `mod/${platform}/run/.minecraft/assets`;
const hasAssets = existsSync(ASSETS_PATH);
const TIMEOUT = hasAssets ? 120 : 300;
const EARLY_TIMEOUT = 60;

const SUCCESS_MARKERS = [
  /Setting user:/,
  /OpenGL:/,
  /Backend library:/,
  /Narrator library:/,
  /Loaded \d+ mods/,
];

const MIXIN_WARNINGS: RegExp[] = [];

let testPassed = false;
let testFailed = false;
let failureReason = "";
let failureExitCode = 1;
let warningCount = 0;
const warnings: string[] = [];
let gameStarted = false;

const buffers: StreamBuffers = { stdout: "", stderr: "" };

const actions: PatternAction[] = [
  {
    patterns: BUILD_ERROR_PATTERNS,
    onMatch: (data) => {
      testFailed = true;
      failureReason = `Build failed:\n${data}`;
      failureExitCode = 1;
      return "stop";
    },
  },
  {
    patterns: FATAL_MIXIN_ERRORS,
    onMatch: (_data, pattern) => {
      testFailed = true;
      failureReason = `Fatal mixin error:\n${extractErrorContext(buffers.stdout + buffers.stderr, pattern)}`;
      failureExitCode = 2;
      return "stop";
    },
  },
  {
    patterns: EARLY_CRASH_PATTERNS,
    onMatch: (_data, pattern) => {
      testFailed = true;
      failureReason = `Early crash detected:\n${extractErrorContext(buffers.stdout + buffers.stderr, pattern)}`;
      failureExitCode = 5;
      return "stop";
    },
  },
  {
    patterns: MIXIN_WARNINGS,
    onMatch: (_data, pattern) => {
      warningCount++;
      const context = extractErrorContext(
        buffers.stdout + buffers.stderr,
        pattern,
      );
      if (!warnings.includes(context)) warnings.push(context);
      return "continue";
    },
  },
  {
    patterns: GAME_STARTED_MARKERS,
    onMatch: () => {
      gameStarted = true;
      return "continue";
    },
  },
  {
    patterns: SUCCESS_MARKERS,
    onMatch: () => {
      testPassed = true;
      return "stop";
    },
  },
];

console.log(c("1;36", `→ Starting Minecraft ${platform} client...`));
console.log(`   Session: ${SESSION_ID}`);

await fs.writeFile(SESSION_FILE, SESSION_ID);
await ensureQuietOptions(platform);

const mc = spawnMinecraft({
  platform,
  env: { ...process.env, GLINT_SESSION: SESSION_ID } as Record<string, string>,
});

const cleanupAll = async () => {
  await mc.cleanup();
  try {
    await fs.unlink(SESSION_FILE);
  } catch {
    /* may not exist */
  }
};

registerSignalHandlers(cleanupAll);

// Start monitoring streams
if (mc.proc.stdout)
  monitorStream(mc.proc.stdout, false, buffers, verbose, actions);
if (mc.proc.stderr)
  monitorStream(mc.proc.stderr, true, buffers, verbose, actions);

console.log(
  c(
    "2",
    `[wait] Timeout: ${EARLY_TIMEOUT}s (early), ${TIMEOUT}s (${hasAssets ? "assets cached" : "first run"})`,
  ),
);

const startTime = Date.now();

try {
  while (true) {
    const elapsedSec = Math.floor((Date.now() - startTime) / 1000);

    if (testPassed) {
      await sleep(2000);
      if (testFailed) {
        console.error(c("31", "[FAIL] Late error detected!"));
        console.error(failureReason);
        await cleanupAll();
        process.exit(failureExitCode);
      }

      console.log(
        c("32", `[PASS] Smoke test passed! (elapsed: ${elapsedSec}s)`),
      );

      if (warningCount > 0) {
        console.warn(c("33", `[WARN] ${warningCount} warning(s) detected`));
        if (verbose) {
          warnings.forEach((warning, idx) => {
            console.warn(`\nWarning ${idx + 1}:`);
            console.warn(warning);
          });
        } else {
          console.warn("   Run with -v to see details");
        }
      }

      await cleanupAll();
      process.exit(0);
    }

    if (testFailed) {
      console.error(c("31", "[FAIL] Test failed!"));
      console.error("");
      console.error(failureReason);
      console.error("");
      if (!verbose) printFailureTail(buffers, 100, "-v or VERBOSE=1", platform);
      await cleanupAll();
      process.exit(failureExitCode);
    }

    const currentTimeout = gameStarted ? TIMEOUT : EARLY_TIMEOUT;
    if (elapsedSec > currentTimeout) {
      console.error(
        c(
          "31",
          `[FAIL] Timeout exceeded (${currentTimeout}s, game ${gameStarted ? "started" : "not started"})!`,
        ),
      );
      console.error("");
      if (verbose) {
        console.error("Full output:");
        console.error(buffers.stdout + buffers.stderr);
      } else {
        printFailureTail(buffers, 100, "-v or VERBOSE=1", platform);
      }
      await cleanupAll();
      process.exit(3);
    }

    if (!isProcessAlive(mc.pid)) {
      console.error(c("31", "[FAIL] Process died unexpectedly!"));
      console.error("");
      if (verbose) {
        if (buffers.stderr) {
          console.error("Full stderr output:");
          console.error(buffers.stderr);
        }
        console.error("Full stdout output:");
        console.error(buffers.stdout);
      } else {
        printFailureTail(buffers, 100, "-v or VERBOSE=1", platform);
      }
      await cleanupAll();
      process.exit(4);
    }

    await sleep(500);
  }
} catch (err) {
  console.error(c("31", `[FAIL] Error during monitoring: ${err}`));
  await cleanupAll();
  process.exit(4);
}
