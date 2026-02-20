#!/usr/bin/env bun

/**
 * Autonomous capture orchestration script.
 *
 * Launches Minecraft in autonomous mode with proper process lifecycle:
 * - Pre-flight: resolves API URL/token, checks backend availability
 * - Spawns Minecraft with GLINT_AUTONOMOUS=true + API env vars
 * - Monitors output for completion, errors, and timeouts
 * - Always terminates the process on completion (success or failure)
 *
 * Usage: bun scripts/orchestrate.ts [flags]
 *
 * Flags:
 *   -p, --platform    Mod platform: fabric (default) or neoforge
 *   -t, --token       Override GLINT_API_TOKEN
 *   -u, --url         Override GLINT_API_URL
 *   -v, --verbose     Show full Minecraft output
 *   -h, --help        Show this help message and exit
 *   --force           Re-capture existing shader×scene combinations
 *   --scenes SEL      Scene filter: slug to capture specific scene
 *   --shaders SEL     Shader filter: slug to capture specific shader
 */

import { existsSync, readFileSync } from "fs";
import { parseFlags, c } from "./lib/fmt";
import {
  GAME_STARTED_MARKERS,
  BUILD_ERROR_PATTERNS,
  FATAL_MIXIN_ERRORS,
  EARLY_CRASH_PATTERNS,
  isProcessAlive,
  sleep,
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
    url: "string",
    token: "string",
    verbose: "bool",
    force: "bool",
    help: "bool",
    scenes: "string",
    shaders: "string",
  } as const,
  {
    p: "platform",
    u: "url",
    t: "token",
    v: "verbose",
    F: "force",
    h: "help",
  },
  {
    platform: "fabric",
    url: "",
    token: "",
    verbose: false,
    force: false,
    help: false,
    scenes: "",
    shaders: "",
  },
);

if (flags.help) {
  console.log(`Usage: bun scripts/orchestrate.ts [flags]

Launches Minecraft in autonomous mode for shader capture orchestration.

Flags:
  -p, --platform <name>   Mod platform: fabric (default) or neoforge
  -t, --token <token>     Override GLINT_API_TOKEN
  -u, --url <url>         Override GLINT_API_URL (default: http://localhost:8080)
  -v, --verbose           Show full Minecraft output
  -h, --help              Show this help message and exit
  -F, --force             Re-capture existing shader×scene combinations
      --scenes <SEL>      Scene filter: slug to capture specific scene (implies --force)
      --shaders <SEL>     Shader filter: slug to capture specific shader (implies --force)

Examples:
  bun scripts/orchestrate.ts                        # Capture needed work
  bun scripts/orchestrate.ts --force                # Re-capture all combinations
  bun scripts/orchestrate.ts --shaders bsl          # Capture BSL shader only
  bun scripts/orchestrate.ts --shaders bsl --scenes meadow  # BSL + meadow scene`);
  process.exit(0);
}

const platform = flags.platform as string;
const verbose = flags.verbose;

if (platform !== "fabric" && platform !== "neoforge") {
  console.error(
    c("31", `Invalid platform: ${platform} (must be 'fabric' or 'neoforge')`),
  );
  process.exit(1);
}

interface ModConfig {
  apiUrl?: string;
  accessToken?: string;
  tokenExpiresAt?: number;
}

function readModConfig(): ModConfig | null {
  const configPath = `mod/${platform}/run/glint/config.json`;
  if (!existsSync(configPath)) return null;
  try {
    return JSON.parse(readFileSync(configPath, "utf-8")) as ModConfig;
  } catch {
    return null;
  }
}

function resolveApiToken(): string {
  if (flags.token) return flags.token as string;
  if (process.env.GLINT_API_TOKEN) return process.env.GLINT_API_TOKEN;

  const config = readModConfig();
  if (config?.accessToken) {
    if (config.tokenExpiresAt && config.tokenExpiresAt < Date.now()) {
      console.error(
        c(
          "31",
          "[preflight] Saved token has expired — re-authenticate via the mod's config wizard",
        ),
      );
      process.exit(1);
    }
    console.log(c("36", "[preflight] Using saved token from mod config"));
    return config.accessToken;
  }

  console.error(c("31", "[preflight] No API token found"));
  console.error("   Provide a token via one of:");
  console.error("     --token <token>           CLI flag");
  console.error("     GLINT_API_TOKEN=<token>   environment variable");
  console.error(
    "     Device auth wizard        (run Minecraft and use the Glint config screen)",
  );
  process.exit(1);
}

function resolveApiUrl(): string {
  if (flags.url) return flags.url as string;
  if (process.env.GLINT_API_URL) return process.env.GLINT_API_URL;

  const config = readModConfig();
  if (config?.apiUrl) return config.apiUrl;

  return "http://localhost:8080";
}

const apiUrl = resolveApiUrl();
const apiToken = resolveApiToken();

async function checkBackend(): Promise<boolean> {
  console.log(`[preflight] Checking backend at ${apiUrl}...`);
  try {
    const resp = await fetch(`${apiUrl}/api/health`, {
      signal: AbortSignal.timeout(5000),
    });
    if (resp.ok) {
      console.log(c("32", "[preflight] ✓ Backend reachable"));
      return true;
    }
    console.error(
      c("31", `[preflight] ✗ Backend returned HTTP ${resp.status}`),
    );
    return false;
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    console.error(c("31", `[preflight] ✗ Backend unreachable at ${apiUrl}`));
    console.error(`   ${msg}`);
    console.error("   Start the backend with: just dev -b");
    return false;
  }
}

if (!(await checkBackend())) {
  process.exit(1);
}

const TERMINAL_MARKERS = [
  /Autonomous runner shutting down/,
  /No work available, shutting down/,
  /Cannot start autonomous mode/,
];

const PROGRESS_MARKERS = [
  // Runner lifecycle
  { pattern: /Autonomous runner started/, msg: "Runner started" },
  { pattern: /Fetched work/, msg: "Work fetched" },
  { pattern: /Created capture run/, msg: "Capture run created" },
  { pattern: /Force mode:/, msg: "Force mode active" },
  { pattern: /No work available/, msg: "No work available" },
  // Asset preparation
  { pattern: /Asset preparation complete/, msg: "Assets prepared" },
  { pattern: /Downloading shader/, msg: "Downloading shader" },
  { pattern: /Downloading scene package/, msg: "Downloading scene package" },
  { pattern: /Scene package downloaded/, msg: "Scene package downloaded" },
  // Staging world
  { pattern: /Starting staging world preparation/, msg: "Preparing staging world" },
  { pattern: /Staging world ready/, msg: "Staging world ready" },
  { pattern: /Staging world created and loaded/, msg: "Staging world created" },
  { pattern: /Staging world loaded/, msg: "Staging world loaded" },
  // Scene & shader loading
  { pattern: /Starting linear orchestration/, msg: "Orchestration starting" },
  { pattern: /Linear capture plan ready/, msg: "Capture plan ready" },
  { pattern: /Loading scene package/, msg: "Loading scene" },
  { pattern: /Scene injection complete/, msg: "Scene injected" },
  { pattern: /Loading shader/, msg: "Loading shader" },
  { pattern: /Shader pack changed/, msg: "Shader switched" },
  { pattern: /Shader profile changed/, msg: "Shader profile changed" },
  { pattern: /Preset applied/, msg: "Preset applied" },
  // Stabilization
  { pattern: /Stabilization phase:/, msg: "Stabilizing" },
  { pattern: /Stabilization complete/, msg: "Stabilization complete" },
  // Capture execution
  { pattern: /Taking capture/, msg: "Taking capture" },
  { pattern: /Starting high-res capture/, msg: "High-res capture started" },
  { pattern: /Capture session begun/, msg: "Capture session started" },
  { pattern: /Capture saved/, msg: "Capture saved" },
  { pattern: /WebP screenshot saved/, msg: "Screenshot saved" },
  { pattern: /Generating manifest/, msg: "Generating manifest" },
  // Upload
  { pattern: /Uploading to R2/, msg: "Uploading" },
  { pattern: /Upload succeeded/, msg: "Upload succeeded" },
  { pattern: /RunUploader drained/, msg: "Uploads drained" },
  { pattern: /Capture run finalized/, msg: "Run finalized" },
  // Completion
  { pattern: /Orchestration complete/, msg: "Orchestration complete" },
  { pattern: /Group upload complete/, msg: "Upload complete" },
  { pattern: /Group upload failed/, msg: "Upload failed" },
];

const RUN_ERROR_MARKERS = [
  /Failed to upload\/complete item/,
  /Failed to fetch work/,
  /Failed to create capture run/,
  /Error preparing capture/,
  /Failed to start orchestration/,
  /Failed to download shader/,
  /Failed to download any worlds/,
  /Group upload failed/,
];

let terminated = false;
let terminateReason = "";
let terminateCode = 0;
let gameStarted = false;
let runFailed = false;
let lastActivity = Date.now();

const buffers: StreamBuffers = { stdout: "", stderr: "" };

// Combine shared fatal patterns with orchestration-specific ones
const FATAL_PATTERNS = [
  ...BUILD_ERROR_PATTERNS,
  ...FATAL_MIXIN_ERRORS,
  ...EARLY_CRASH_PATTERNS,
];

const actions: PatternAction[] = [
  {
    patterns: FATAL_PATTERNS,
    onMatch: (data) => {
      terminated = true;
      terminateReason = `Fatal error: ${data.trim().slice(0, 200)}`;
      terminateCode = 1;
      return "stop";
    },
  },
  {
    patterns: GAME_STARTED_MARKERS,
    onMatch: () => {
      if (!gameStarted) {
        gameStarted = true;
        lastActivity = Date.now();
        if (!verbose) console.log(c("36", "[orchestrate] Game loaded"));
      }
      return "continue";
    },
  },
  {
    patterns: PROGRESS_MARKERS.map((m) => m.pattern),
    onMatch: (data) => {
      lastActivity = Date.now();
      if (!verbose) {
        for (const { pattern, msg } of PROGRESS_MARKERS) {
          if (pattern.test(data)) {
            console.log(c("36", `[orchestrate] ${msg}`));
          }
        }
      }
      return "continue";
    },
  },
  {
    patterns: RUN_ERROR_MARKERS,
    onMatch: (data) => {
      runFailed = true;
      if (!verbose) {
        for (const line of data.split("\n")) {
          for (const pattern of RUN_ERROR_MARKERS) {
            if (pattern.test(line)) {
              console.error(
                c("31", `[orchestrate] ${line.trim().slice(0, 200)}`),
              );
            }
          }
        }
      }
      return "continue";
    },
  },
  {
    patterns: TERMINAL_MARKERS,
    onMatch: () => {
      terminated = true;
      terminateReason = "Orchestration finished";
      terminateCode = 0;
      return "stop";
    },
  },
];

await ensureQuietOptions(platform);

const forceMode = flags.force || !!flags.scenes || !!flags.shaders;
const forceScenes = (flags.scenes as string) || undefined;
const forceShaders = (flags.shaders as string) || undefined;

if (forceMode) {
  console.log(
    c(
      "1;33",
      `→ Force mode: scenes=${forceScenes ?? "all"}, shaders=${forceShaders ?? "all"}`,
    ),
  );
}
console.log(
  c("1;36", `→ Launching Minecraft (${platform}) in autonomous mode...`),
);
console.log(`   API: ${apiUrl}`);
console.log(`   Token: ${apiToken.slice(0, 4)}...`);

const modEnv: Record<string, string> = {
  ...Object.fromEntries(
    Object.entries(process.env).filter(
      (entry): entry is [string, string] => entry[1] != null,
    ),
  ),
  GLINT_AUTONOMOUS: "true",
  GLINT_API_URL: apiUrl,
  GLINT_API_TOKEN: apiToken,
};

if (forceMode) {
  modEnv.GLINT_FORCE_SCENES = forceScenes ?? "+";
  modEnv.GLINT_FORCE_SHADERS = forceShaders ?? "+";
}

const mc = spawnMinecraft({ platform, env: modEnv });
registerSignalHandlers(mc.cleanup);

if (mc.proc.stdout)
  monitorStream(mc.proc.stdout, false, buffers, verbose, actions);
if (mc.proc.stderr)
  monitorStream(mc.proc.stderr, true, buffers, verbose, actions);

const startTime = Date.now();
const EARLY_TIMEOUT = 120;
const INACTIVITY_TIMEOUT = 180;

try {
  while (true) {
    const elapsedSec = Math.floor((Date.now() - startTime) / 1000);

    // Orchestration completed normally
    if (terminated && terminateCode === 0) {
      if (runFailed) {
        console.error(c("31", `[orchestrate] Run failed (${elapsedSec}s)`));
        if (!verbose) printFailureTail(buffers, 30, "-v");
        await mc.cleanup();
        process.exit(1);
      }
      console.log(
        c("32", `[orchestrate] Orchestration complete (${elapsedSec}s)`),
      );
      await sleep(3000);
      await mc.cleanup();
      process.exit(0);
    }

    // Fatal error detected
    if (terminated && terminateCode !== 0) {
      console.error(c("31", `[orchestrate] ${terminateReason}`));
      if (!verbose) printFailureTail(buffers, 50, "-v");
      await mc.cleanup();
      process.exit(terminateCode);
    }

    // Process died
    if (!isProcessAlive(mc.pid)) {
      const exitCode = await mc.proc.exited;
      if (exitCode === 0) {
        console.log(
          c("32", `[orchestrate] Process exited cleanly (${elapsedSec}s)`),
        );
      } else {
        console.error(
          c("31", `[orchestrate] Process exited with code ${exitCode}`),
        );
        if (!verbose) printFailureTail(buffers, 50, "-v");
      }
      process.exit(exitCode);
    }

    // Timeout
    const idleSeconds = Math.floor((Date.now() - lastActivity) / 1000);
    const timedOut = gameStarted
      ? idleSeconds > INACTIVITY_TIMEOUT
      : elapsedSec > EARLY_TIMEOUT;
    if (timedOut) {
      const reason = gameStarted
        ? `No activity for ${idleSeconds}s (limit: ${INACTIVITY_TIMEOUT}s)`
        : `Game did not start within ${EARLY_TIMEOUT}s`;
      console.error(c("31", `[orchestrate] Timeout: ${reason}`));
      if (!verbose) printFailureTail(buffers, 50, "-v");
      await mc.cleanup();
      process.exit(3);
    }

    await sleep(500);
  }
} catch (err) {
  console.error(c("31", `[orchestrate] Monitoring error: ${err}`));
  await mc.cleanup();
  process.exit(1);
}
