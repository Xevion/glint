/**
 * Shared Minecraft process management for smoke tests and orchestration.
 *
 * Provides common utilities for launching Minecraft via Gradle, monitoring
 * output streams for error/success patterns, managing process lifecycle,
 * and ensuring clean shutdown.
 */

import { spawn, type Subprocess } from "bun";
import * as fs from "fs/promises";
import { existsSync } from "fs";
import { c } from "./fmt";

/** Patterns indicating the game has begun loading (but not yet fully started). */
export const GAME_STARTED_MARKERS = [
  /Loaded \d+ mods/,
  /FabricLoader\//,
  /Loading Minecraft/,
  /Mixin Subsystem/,
  /NeoForge/,
  /Architectury/,
];

/** Build-time errors that indicate Gradle/compiler failure. */
export const BUILD_ERROR_PATTERNS = [
  /BUILD FAILED/,
  /compilation failed/i,
  /error: cannot find symbol/i,
  /Execution failed for task/i,
];

/** Fatal mixin errors — the game cannot recover from these. */
export const FATAL_MIXIN_ERRORS = [
  /critical injection/i,
  /@Shadow.*not located/i,
  /transformation.*failed/i,
  /mixin apply.*failed/i,
  /Target method.*not found/i,
  /Mixin.*crashed/i,
];

/**
 * Early crash patterns — process will likely die immediately after these.
 *
 * NOTE: ClassNotFoundException/NoClassDefFoundError require "Caused by:" context
 * because the mixin system logs benign warnings when probing optional classes.
 */
export const EARLY_CRASH_PATTERNS = [
  /Caused by:.*ClassNotFoundException/,
  /Caused by:.*NoClassDefFoundError/,
  /Exception in thread "main"/,
  /Error: Could not find or load main class/,
];

/** Valid Minecraft mod platform identifiers. */
export type Platform = "fabric" | "neoforge";

/**
 * Validate that a platform string is a valid mod platform.
 * Exits with code 1 and an error message if invalid.
 */
export function validatePlatform(
  platform: string,
): asserts platform is Platform {
  if (platform !== "fabric" && platform !== "neoforge") {
    console.error(
      c("31", `Invalid platform: ${platform} (must be 'fabric' or 'neoforge')`),
    );
    process.exit(1);
  }
}

/** Check whether a process with the given PID is still alive. */
export function isProcessAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/** Promise-based sleep. */
export const sleep = (ms: number) =>
  new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Extract lines surrounding the first match of a pattern in log content.
 * Returns 5 lines before and 5 lines after the match.
 */
export function extractErrorContext(
  logContent: string,
  pattern: RegExp,
): string {
  const lines = logContent.split("\n");
  for (let i = 0; i < lines.length; i++) {
    if (pattern.test(lines[i])) {
      const start = Math.max(0, i - 5);
      const end = Math.min(lines.length, i + 6);
      return lines.slice(start, end).join("\n");
    }
  }
  return "Error context not found";
}

const QUIET_OPTIONS = `soundCategory_master:0.0
soundCategory_music:0.0
soundCategory_record:0.0
soundCategory_weather:0.0
soundCategory_block:0.0
soundCategory_hostile:0.0
soundCategory_neutral:0.0
soundCategory_player:0.0
soundCategory_ambient:0.0
soundCategory_voice:0.0
particles:0
renderDistance:2
graphicsMode:1
chatVisibility:2
narrator:0
showSubtitles:false
fullscreen:false`;

/**
 * Create options.txt with quiet/minimal settings if it doesn't already exist.
 * Does not overwrite existing user settings.
 */
export async function ensureQuietOptions(platform: string): Promise<void> {
  const optionsPath = `mod/${platform}/run/options.txt`;
  if (existsSync(optionsPath)) return;

  await fs.mkdir(`mod/${platform}/run`, { recursive: true });
  await fs.writeFile(optionsPath, QUIET_OPTIONS);
}

/** Configuration for a single pattern match action in stream monitoring. */
export interface PatternAction {
  patterns: RegExp[];
  /** Called when any pattern matches. Return "stop" to end monitoring. */
  onMatch: (data: string, pattern: RegExp) => "stop" | "continue";
}

/** Result of monitoring a stream — captures all buffered output. */
export interface StreamBuffers {
  stdout: string;
  stderr: string;
}

/**
 * Monitor an output stream (stdout or stderr), checking chunks against
 * a list of pattern actions.
 *
 * Appends all data to the appropriate buffer. If verbose is true, data
 * is also written to the corresponding process stream.
 */
export async function monitorStream(
  stream: ReadableStream,
  isStderr: boolean,
  buffers: StreamBuffers,
  verbose: boolean,
  actions: PatternAction[],
): Promise<void> {
  try {
    const reader = stream.getReader();
    const decoder = new TextDecoder();

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const data = decoder.decode(value, { stream: true });

      if (isStderr) {
        buffers.stderr += data;
      } else {
        buffers.stdout += data;
      }

      if (verbose) {
        if (isStderr) {
          process.stderr.write(data);
        } else {
          process.stdout.write(data);
        }
      }

      for (const action of actions) {
        for (const pattern of action.patterns) {
          if (pattern.test(data)) {
            const result = action.onMatch(data, pattern);
            if (result === "stop") return;
          }
        }
      }
    }
  } catch {
    // Stream closed — normal on process exit
  }
}

/** Options for spawning and managing a Minecraft process. */
export interface MinecraftProcessOptions {
  platform: string;
  gradleArgs?: string[];
  env?: Record<string, string>;
}

/** Handle for a managed Minecraft process with cleanup support. */
export interface MinecraftProcess {
  proc: Subprocess;
  pid: number;
  cleanup: () => Promise<void>;
}

/**
 * Spawn a Minecraft client via Gradle with proper process lifecycle.
 *
 * Returns the process handle, PID, and a cleanup function that
 * gracefully terminates with SIGTERM → wait → SIGKILL escalation.
 */
export function spawnMinecraft(
  options: MinecraftProcessOptions,
): MinecraftProcess {
  const defaultArgs = [
    `:${options.platform}:runClient`,
    "--no-daemon",
    "--args=--nogui --width=854 --height=480",
  ];
  const args = options.gradleArgs ?? defaultArgs;

  const proc = spawn(["./gradlew", ...args], {
    cwd: "mod",
    stdio: ["ignore", "pipe", "pipe"],
    env: options.env ?? { ...process.env },
  });

  const pid = proc.pid!;
  let cleanupDone = false;

  const cleanup = async () => {
    if (cleanupDone) return;
    cleanupDone = true;

    if (isProcessAlive(pid)) {
      try {
        process.kill(pid, "SIGTERM");
        await sleep(5000);

        if (isProcessAlive(pid)) {
          process.kill(pid, "SIGKILL");
        }

        // Kill process group
        try {
          process.kill(-pid, "SIGTERM");
        } catch {
          // Process group may not exist
        }
      } catch {
        // Process already dead
      }
    }
  };

  return { proc, pid, cleanup };
}

/**
 * Register signal handlers that run cleanup on SIGINT/SIGTERM.
 *
 * Returns a function to remove the handlers (for use after cleanup).
 */
export function registerSignalHandlers(
  cleanup: () => Promise<void>,
): () => void {
  const onSigint = async () => {
    await cleanup();
    process.exit(130);
  };
  const onSigterm = async () => {
    await cleanup();
    process.exit(143);
  };

  process.on("SIGINT", onSigint);
  process.on("SIGTERM", onSigterm);
  process.on("exit", () => {
    // Best-effort synchronous cleanup on exit
    cleanup();
  });

  return () => {
    process.off("SIGINT", onSigint);
    process.off("SIGTERM", onSigterm);
  };
}

/**
 * Print the last N lines of output, with a hint to run with verbose mode.
 * Used by both smoke.ts and orchestrate.ts for failure output.
 */
export function printFailureTail(
  buffers: StreamBuffers,
  lines: number,
  verboseHint: string,
  platform?: string,
): void {
  const allOutput = buffers.stdout + buffers.stderr;
  console.error(`Last ${lines} lines of output:`);
  console.error(allOutput.split("\n").slice(-lines).join("\n"));
  console.error("");
  console.error(`[hint] Run with ${verboseHint} for full output`);
  if (platform) {
    console.error("");
    console.error("[hint] Or check the log file for errors:");
    console.error(
      `   grep -i "error\\|exception\\|fatal" mod/${platform}/run/logs/latest.log | tail -50`,
    );
  }
}

/** Configuration for the unified monitor loop. */
export interface MonitorLoopConfig {
  /** Minecraft process handle. */
  mc: MinecraftProcess;
  /** Cleanup function called before exit. */
  cleanup: () => Promise<void>;
  /** Polling interval in ms (default: 500). */
  pollInterval?: number;
  /** Label for the catch-block error message. */
  label?: string;

  /** Return true when the operation has completed successfully. */
  isComplete: () => boolean;
  /** Handle successful completion. Return the process exit code. */
  onComplete: (elapsedSec: number) => Promise<number>;

  /** Return true when a fatal error has been detected. */
  isFailed: () => boolean;
  /** Handle failure. Return the process exit code. */
  onFailed: (elapsedSec: number) => Promise<number>;

  /** Check if any timeout has been exceeded. Return a reason string, or null. */
  checkTimeout: (elapsedSec: number) => string | null;
  /** Handle timeout. Return the process exit code. */
  onTimeout: (reason: string, elapsedSec: number) => Promise<number>;

  /** Handle unexpected process death. Return the process exit code. */
  onProcessDied: (elapsedSec: number) => Promise<number>;
}

/**
 * Unified polling monitor loop for Minecraft process management.
 *
 * Polls at `pollInterval` (default 500ms), checking in order:
 * isComplete → isFailed → checkTimeout → process death.
 *
 * Each handler returns an exit code; the loop calls cleanup() then
 * process.exit() with that code. Never returns.
 */
export async function runMonitorLoop(
  config: MonitorLoopConfig,
): Promise<never> {
  const {
    mc,
    cleanup,
    pollInterval = 500,
    label = "monitor",
    isComplete,
    onComplete,
    isFailed,
    onFailed,
    checkTimeout,
    onTimeout,
    onProcessDied,
  } = config;

  const startTime = Date.now();

  try {
    while (true) {
      const elapsedSec = Math.floor((Date.now() - startTime) / 1000);

      if (isComplete()) {
        const code = await onComplete(elapsedSec);
        await cleanup();
        process.exit(code);
      }

      if (isFailed()) {
        const code = await onFailed(elapsedSec);
        await cleanup();
        process.exit(code);
      }

      const timeoutReason = checkTimeout(elapsedSec);
      if (timeoutReason !== null) {
        const code = await onTimeout(timeoutReason, elapsedSec);
        await cleanup();
        process.exit(code);
      }

      if (!isProcessAlive(mc.pid)) {
        const code = await onProcessDied(elapsedSec);
        await cleanup();
        process.exit(code);
      }

      await sleep(pollInterval);
    }
  } catch (err) {
    console.error(c("31", `[${label}] Monitoring error: ${err}`));
    await cleanup();
    process.exit(1);
  }
}
