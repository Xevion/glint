#!/usr/bin/env bun

/**
 * Autonomous capture orchestration script.
 *
 * Launches Minecraft in autonomous mode with proper process lifecycle:
 * - Pre-flight: resolves API URL/token, checks backend availability
 * - Spawns Minecraft with GLINT_AUTONOMOUS=true + API env vars
 * - Monitors output for completion, errors, and timeouts
 * - Always terminates the process on completion (success or failure)
 */

import { defineCommand } from "@xevion/tempo";
import { c } from "@xevion/tempo/fmt";
import { existsSync, readFileSync } from "fs";
import {
  GAME_STARTED_MARKERS,
  BUILD_ERROR_PATTERNS,
  FATAL_MIXIN_ERRORS,
  EARLY_CRASH_PATTERNS,
  sleep,
  ensureQuietOptions,
  monitorStream,
  spawnMinecraft,
  registerSignalHandlers,
  printFailureTail,
  validatePlatform,
  runMonitorLoop,
  type StreamBuffers,
  type PatternAction,
} from "./minecraft.ts";

export default defineCommand(
  {
    name: "orchestrate",
    description: "Run autonomous shader capture session",
    flags: {
      platform: { type: String, alias: "p", description: "Mod platform: fabric or neoforge", default: "fabric" },
      url: { type: String, alias: "u", description: "Override GLINT_API_URL", default: "" },
      token: { type: String, alias: "t", description: "Override GLINT_API_TOKEN", default: "" },
      verbose: { type: Boolean, alias: "v", description: "Show full Minecraft output" },
      force: { type: Boolean, description: "Re-capture existing shader×scene combinations" },
      scenes: { type: String, description: "Scene filter: slug to capture specific scene", default: "" },
      shaders: { type: String, description: "Shader filter: slug to capture specific shader", default: "" },
      help: { type: Boolean, alias: "h", description: "Show help" },
    },
    run: async (ctx) => {
      if (ctx.flags.help) {
        console.log(`Usage: bun scripts/orchestrate.ts [flags]

Launches Minecraft in autonomous mode for shader capture orchestration.

Flags:
  -p, --platform <name>   Mod platform: fabric (default) or neoforge
  -t, --token <token>     Override GLINT_API_TOKEN
  -u, --url <url>         Override GLINT_API_URL (default: http://localhost:8080)
  -v, --verbose           Show full Minecraft output
  -h, --help              Show this help message and exit
      --force             Re-capture existing shader×scene combinations
      --scenes <SEL>      Scene filter: slug to capture specific scene (implies --force)
      --shaders <SEL>     Shader filter: slug to capture specific shader (implies --force)

Examples:
  bun scripts/orchestrate.ts                        # Capture needed work
  bun scripts/orchestrate.ts --force                # Re-capture all combinations
  bun scripts/orchestrate.ts --shaders bsl          # Capture BSL shader only
  bun scripts/orchestrate.ts --shaders bsl --scenes meadow  # BSL + meadow scene`);
        return 0;
      }

      const platform = ctx.flags.platform;
      const verbose = ctx.flags.verbose;

      validatePlatform(platform);

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
        if (ctx.flags.token) return ctx.flags.token;
        if (process.env.GLINT_API_TOKEN) return process.env.GLINT_API_TOKEN;

        const config = readModConfig();
        if (config?.accessToken) {
          if (config.tokenExpiresAt && config.tokenExpiresAt < Date.now()) {
            console.error(
              c.red(
                "[preflight] Saved token has expired — re-authenticate via the mod's config wizard",
              ),
            );
            process.exit(1);
          }
          console.log(c.cyan("[preflight] Using saved token from mod config"));
          return config.accessToken;
        }

        console.error(c.red("[preflight] No API token found"));
        console.error("   Provide a token via one of:");
        console.error("     --token <token>           CLI flag");
        console.error("     GLINT_API_TOKEN=<token>   environment variable");
        console.error(
          "     Device auth wizard        (run Minecraft and use the Glint config screen)",
        );
        process.exit(1);
      }

      function resolveApiUrl(): string {
        if (ctx.flags.url) return ctx.flags.url;
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
            console.log(c.green("[preflight] ✓ Backend reachable"));
            return true;
          }
          console.error(
            c.red(`[preflight] ✗ Backend returned HTTP ${resp.status}`),
          );
          return false;
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err);
          console.error(c.red(`[preflight] ✗ Backend unreachable at ${apiUrl}`));
          console.error(`   ${msg}`);
          console.error("   Start the backend with: just dev -b");
          return false;
        }
      }

      if (!(await checkBackend())) {
        return 1;
      }

      const TERMINAL_MARKERS = [
        /Autonomous runner shutting down/,
        /No work available, shutting down/,
        /Cannot start autonomous mode/,
      ];

      const PROGRESS_MARKERS = [
        { pattern: /Autonomous runner started/, msg: "Runner started" },
        { pattern: /Fetched work/, msg: "Work fetched" },
        { pattern: /Created capture run/, msg: "Capture run created" },
        { pattern: /Force mode:/, msg: "Force mode active" },
        { pattern: /No work available/, msg: "No work available" },
        { pattern: /Asset preparation complete/, msg: "Assets prepared" },
        { pattern: /Downloading shader/, msg: "Downloading shader" },
        { pattern: /Downloading scene package/, msg: "Downloading scene package" },
        { pattern: /Scene package downloaded/, msg: "Scene package downloaded" },
        { pattern: /Starting staging world preparation/, msg: "Preparing staging world" },
        { pattern: /Staging world ready/, msg: "Staging world ready" },
        { pattern: /Staging world created and loaded/, msg: "Staging world created" },
        { pattern: /Staging world loaded/, msg: "Staging world loaded" },
        { pattern: /Starting linear orchestration/, msg: "Orchestration starting" },
        { pattern: /Linear capture plan ready/, msg: "Capture plan ready" },
        { pattern: /Loading scene package/, msg: "Loading scene" },
        { pattern: /Scene injection complete/, msg: "Scene injected" },
        { pattern: /Loading shader/, msg: "Loading shader" },
        { pattern: /Shader pack changed/, msg: "Shader switched" },
        { pattern: /Shader profile changed/, msg: "Shader profile changed" },
        { pattern: /Preset applied/, msg: "Preset applied" },
        { pattern: /Stabilization phase:/, msg: "Stabilizing" },
        { pattern: /Stabilization complete/, msg: "Stabilization complete" },
        { pattern: /Taking capture/, msg: "Taking capture" },
        { pattern: /Starting high-res capture/, msg: "High-res capture started" },
        { pattern: /Capture session begun/, msg: "Capture session started" },
        { pattern: /Capture saved/, msg: "Capture saved" },
        { pattern: /WebP screenshot saved/, msg: "Screenshot saved" },
        { pattern: /Generating manifest/, msg: "Generating manifest" },
        { pattern: /Uploading to R2/, msg: "Uploading" },
        { pattern: /Upload succeeded/, msg: "Upload succeeded" },
        { pattern: /RunUploader drained/, msg: "Uploads drained" },
        { pattern: /Capture run finalized/, msg: "Run finalized" },
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
              if (!verbose) console.log(c.cyan("[orchestrate] Game loaded"));
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
                  console.log(c.cyan(`[orchestrate] ${msg}`));
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
                      c.red(`[orchestrate] ${line.trim().slice(0, 200)}`),
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

      const forceMode = ctx.flags.force || !!ctx.flags.scenes || !!ctx.flags.shaders;
      const forceScenes = ctx.flags.scenes || undefined;
      const forceShaders = ctx.flags.shaders || undefined;

      if (forceMode) {
        console.log(
          c.bold.yellow(
            `→ Force mode: scenes=${forceScenes ?? "all"}, shaders=${forceShaders ?? "all"}`,
          ),
        );
      }
      console.log(
        c.bold.cyan(`→ Launching Minecraft (${platform}) in autonomous mode...`),
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

      const EARLY_TIMEOUT = 120;
      const INACTIVITY_TIMEOUT = 180;

      await runMonitorLoop({
        mc,
        cleanup: mc.cleanup,
        label: "orchestrate",

        isComplete: () => terminated && terminateCode === 0,
        onComplete: async (elapsedSec) => {
          if (runFailed) {
            console.error(c.red(`[orchestrate] Run failed (${elapsedSec}s)`));
            if (!verbose) printFailureTail(buffers, 30, "-v");
            return 1;
          }
          console.log(
            c.green(`[orchestrate] Orchestration complete (${elapsedSec}s)`),
          );
          await sleep(3000);
          return 0;
        },

        isFailed: () => terminated && terminateCode !== 0,
        onFailed: async () => {
          console.error(c.red(`[orchestrate] ${terminateReason}`));
          if (!verbose) printFailureTail(buffers, 50, "-v");
          return terminateCode;
        },

        checkTimeout: (elapsedSec) => {
          if (gameStarted) {
            const idleSeconds = Math.floor((Date.now() - lastActivity) / 1000);
            if (idleSeconds > INACTIVITY_TIMEOUT) {
              return `No activity for ${idleSeconds}s (limit: ${INACTIVITY_TIMEOUT}s)`;
            }
          } else if (elapsedSec > EARLY_TIMEOUT) {
            return `Game did not start within ${EARLY_TIMEOUT}s`;
          }
          return null;
        },
        onTimeout: async (reason) => {
          console.error(c.red(`[orchestrate] Timeout: ${reason}`));
          if (!verbose) printFailureTail(buffers, 50, "-v");
          return 3;
        },

        onProcessDied: async (elapsedSec) => {
          const exitCode = await mc.exited;
          if (exitCode === 0) {
            console.log(
              c.green(`[orchestrate] Process exited cleanly (${elapsedSec}s)`),
            );
          } else {
            console.error(
              c.red(`[orchestrate] Process exited with code ${exitCode}`),
            );
            if (!verbose) printFailureTail(buffers, 50, "-v");
          }
          return exitCode;
        },
      });

      return 0; // unreachable — runMonitorLoop never returns
    },
  },
  import.meta.main,
);
