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

import { spawn, type Subprocess } from "bun";
import * as fs from "fs/promises";
import { existsSync, readFileSync } from "fs";
import { parseFlags, c } from "./lib/fmt";

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
	console.error(c("31", `Invalid platform: ${platform} (must be 'fabric' or 'neoforge')`));
	process.exit(1);
}

// ── Resolve API configuration ──

interface ModConfig {
	apiUrl?: string;
	accessToken?: string;
	tokenExpiresAt?: number;
}

/** Read the mod's saved config from .minecraft/glint/config.json */
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
	// 1. CLI flag
	if (flags.token) return flags.token as string;

	// 2. Environment variable
	if (process.env.GLINT_API_TOKEN) return process.env.GLINT_API_TOKEN;

	// 3. Saved mod config (from device auth flow)
	const config = readModConfig();
	if (config?.accessToken) {
		if (config.tokenExpiresAt && config.tokenExpiresAt < Date.now()) {
			console.error(c("31", "[preflight] Saved token has expired — re-authenticate via the mod's config wizard"));
			process.exit(1);
		}
		console.log(c("36", "[preflight] Using saved token from mod config"));
		return config.accessToken;
	}

	console.error(c("31", "[preflight] No API token found"));
	console.error("   Provide a token via one of:");
	console.error("     --token <token>           CLI flag");
	console.error("     GLINT_API_TOKEN=<token>   environment variable");
	console.error("     Device auth wizard        (run Minecraft and use the Glint config screen)");
	process.exit(1);
}

function resolveApiUrl(): string {
	if (flags.url) return flags.url as string;
	if (process.env.GLINT_API_URL) return process.env.GLINT_API_URL;

	// Fall back to mod config URL if available
	const config = readModConfig();
	if (config?.apiUrl) return config.apiUrl;

	return "http://localhost:8080";
}

const apiUrl = resolveApiUrl();
const apiToken = resolveApiToken();

// ── Pre-flight checks ──

async function checkBackend(): Promise<boolean> {
	console.log(`[preflight] Checking backend at ${apiUrl}...`);
	try {
		const resp = await fetch(`${apiUrl}/health`, {
			signal: AbortSignal.timeout(5000),
		});
		if (resp.ok) {
			console.log(c("32", `[preflight] ✓ Backend reachable`));
			return true;
		}
		console.error(c("31", `[preflight] ✗ Backend returned HTTP ${resp.status}`));
		return false;
	} catch (err: unknown) {
		const msg = err instanceof Error ? err.message : String(err);
		console.error(c("31", `[preflight] ✗ Backend unreachable at ${apiUrl}`));
		console.error(`   ${msg}`);
		console.error(`   Start the backend with: just dev -b`);
		return false;
	}
}

if (!(await checkBackend())) {
	process.exit(1);
}

// ── Process lifecycle ──

let proc: Subprocess | null = null;
let procPid: number | null = null;
let cleanupDone = false;

function isProcessAlive(pid: number): boolean {
	try {
		process.kill(pid, 0);
		return true;
	} catch {
		return false;
	}
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function cleanup(): Promise<void> {
	if (cleanupDone) return;
	cleanupDone = true;

	if (procPid && isProcessAlive(procPid)) {
		console.log("[cleanup] Stopping Minecraft process...");
		try {
			process.kill(procPid, "SIGTERM");
			await sleep(5000);

			if (isProcessAlive(procPid)) {
				console.log("[cleanup] Force killing...");
				process.kill(procPid, "SIGKILL");
			}

			// Kill process group
			try {
				process.kill(-procPid, "SIGTERM");
			} catch {
				// Process group may not exist
			}
		} catch {
			// Process already dead
		}
	}
}

process.on("exit", () => {
	if (!cleanupDone) cleanup();
});
process.on("SIGINT", async () => {
	console.log("\n[signal] Received SIGINT, cleaning up...");
	await cleanup();
	process.exit(130);
});
process.on("SIGTERM", async () => {
	await cleanup();
	process.exit(143);
});

// ── Output monitoring ──

// Terminal markers - orchestration is done (game should be exiting)
const TERMINAL_MARKERS = [
	/Autonomous runner shutting down/,
	/No work available, shutting down/,
	/Cannot start autonomous mode/,
];

// Error markers that indicate the game won't recover
const FATAL_PATTERNS = [
	/BUILD FAILED/,
	/compilation failed/i,
	/Execution failed for task/i,
	/critical injection/i,
	/mixin apply.*failed/i,
	/Caused by:.*ClassNotFoundException/,
	/Caused by:.*NoClassDefFoundError/,
	/Exception in thread "main"/,
];

// Informational markers
const PROGRESS_MARKERS = [
	{ pattern: /Autonomous runner started/, msg: "Runner started" },
	{ pattern: /Fetched work/, msg: "Work fetched" },
	{ pattern: /Created capture run/, msg: "Capture run created" },
	{ pattern: /Force mode:/, msg: "Force mode active" },
	{ pattern: /Starting capture/, msg: "Capture starting" },
	{ pattern: /Orchestration complete/, msg: "Capture complete" },
	{ pattern: /Capture run finalized/, msg: "Run finalized" },
	{ pattern: /Group upload complete/, msg: "Upload complete" },
	{ pattern: /Group upload failed/, msg: "Upload failed" },
	{ pattern: /No work available/, msg: "No work available" },
];

let outputBuffer = "";
let stderrBuffer = "";
let terminated = false;
let terminateReason = "";
let terminateCode = 0;
let gameStarted = false;
let runFailed = false;

// Patterns that indicate work failed (not fatal to the process, but the work wasn't done)
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

const GAME_STARTED_MARKERS = [
	/Loaded \d+ mods/,
	/FabricLoader\//,
	/Loading Minecraft/,
	/NeoForge/,
	/Architectury/,
];

async function monitorStream(stream: ReadableStream, isStderr: boolean): Promise<void> {
	try {
		const reader = stream.getReader();
		const decoder = new TextDecoder();

		while (true) {
			const { done, value } = await reader.read();
			if (done) break;

			const data = decoder.decode(value, { stream: true });

			if (isStderr) {
				stderrBuffer += data;
			} else {
				outputBuffer += data;
			}

			if (verbose) {
				if (isStderr) {
					process.stderr.write(data);
				} else {
					process.stdout.write(data);
				}
			}

			// Check for fatal errors (fail fast)
			for (const pattern of FATAL_PATTERNS) {
				if (pattern.test(data)) {
					terminated = true;
					terminateReason = `Fatal error: ${data.trim().slice(0, 200)}`;
					terminateCode = 1;
					return;
				}
			}

			// Check for game started markers
			if (!gameStarted) {
				for (const marker of GAME_STARTED_MARKERS) {
					if (marker.test(data)) {
						gameStarted = true;
						if (!verbose) {
							console.log(c("36", "[orchestrate] Game loaded"));
						}
						break;
					}
				}
			}

			// Check progress markers (informational, don't terminate)
			if (!verbose) {
				for (const { pattern, msg } of PROGRESS_MARKERS) {
					if (pattern.test(data)) {
						console.log(c("36", `[orchestrate] ${msg}`));
					}
				}
			}

			// Check for run-level errors
			for (const pattern of RUN_ERROR_MARKERS) {
				if (pattern.test(data)) {
					runFailed = true;
					if (!verbose) {
						for (const line of data.split("\n")) {
							if (pattern.test(line)) {
								console.error(c("31", `[orchestrate] ${line.trim().slice(0, 200)}`));
							}
						}
					}
					break;
				}
			}

			// Check terminal markers (orchestration finished, wait for process to exit)
			for (const pattern of TERMINAL_MARKERS) {
				if (pattern.test(data)) {
					terminated = true;
					terminateReason = "Orchestration finished";
					terminateCode = 0;
					return;
				}
			}
		}
	} catch {
		// Stream closed - normal on process exit
	}
}

// ── Spawn and monitor ──

// Create quiet options if needed
const OPTIONS_PATH = `mod/${platform}/run/options.txt`;
if (!existsSync(OPTIONS_PATH)) {
	await fs.mkdir(`mod/${platform}/run`, { recursive: true });
	await fs.writeFile(
		OPTIONS_PATH,
		`soundCategory_master:0.0
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
fullscreen:false`,
	);
}

// Resolve force mode — --scenes or --shaders implies --force
const forceMode = flags.force || !!flags.scenes || !!flags.shaders;
const forceScenes = (flags.scenes as string) || undefined;
const forceShaders = (flags.shaders as string) || undefined;

if (forceMode) {
	console.log(c("1;33", `→ Force mode: scenes=${forceScenes ?? "all"}, shaders=${forceShaders ?? "all"}`));
}
console.log(c("1;36", `→ Launching Minecraft (${platform}) in autonomous mode...`));
console.log(`   API: ${apiUrl}`);
console.log(`   Token: ${apiToken.slice(0, 4)}...`);

const gradleArgs = [`:${platform}:runClient`, "--no-daemon", "--args=--nogui --width=854 --height=480"];

const modEnv: Record<string, string> = {
	...process.env as Record<string, string>,
	GLINT_AUTONOMOUS: "true",
	GLINT_API_URL: apiUrl,
	GLINT_API_TOKEN: apiToken,
};

if (forceMode) {
	modEnv.GLINT_FORCE_SCENES = forceScenes ?? "+";
	modEnv.GLINT_FORCE_SHADERS = forceShaders ?? "+";
}

proc = spawn(["./gradlew", ...gradleArgs], {
	cwd: "mod",
	stdio: ["ignore", "pipe", "pipe"],
	env: modEnv,
});
procPid = proc.pid!;

if (proc.stdout) monitorStream(proc.stdout, false);
if (proc.stderr) monitorStream(proc.stderr, true);

// Wait loop
const startTime = Date.now();
const EARLY_TIMEOUT = 120; // 2 min for game to start
const FULL_TIMEOUT = 600; // 10 min total for orchestration

try {
	while (true) {
		const elapsed = Math.floor((Date.now() - startTime) / 1000);

		// Orchestration completed normally
		if (terminated && terminateCode === 0) {
			if (runFailed) {
				console.error(c("31", `[orchestrate] Run failed (${elapsed}s)`));
				if (!verbose) {
					const allOutput = outputBuffer + stderrBuffer;
					console.error("");
					console.error("Last 30 lines of output:");
					console.error(allOutput.split("\n").slice(-30).join("\n"));
					console.error("");
					console.error(`[hint] Run with -v for full output`);
				}
				await cleanup();
				process.exit(1);
			}
			// Give the process a moment to exit cleanly
			console.log(c("32", `[orchestrate] Orchestration complete (${elapsed}s)`));
			await sleep(3000);
			await cleanup();
			process.exit(0);
		}

		// Fatal error detected
		if (terminated && terminateCode !== 0) {
			console.error(c("31", `[orchestrate] ${terminateReason}`));
			if (!verbose) {
				console.error("");
				console.error("Last 50 lines of output:");
				const allOutput = outputBuffer + stderrBuffer;
				console.error(allOutput.split("\n").slice(-50).join("\n"));
				console.error("");
				console.error(`[hint] Run with -v for full output`);
			}
			await cleanup();
			process.exit(terminateCode);
		}

		// Process died
		if (procPid && !isProcessAlive(procPid)) {
			const exitCode = await proc!.exited;
			if (exitCode === 0) {
				console.log(c("32", `[orchestrate] Process exited cleanly (${elapsed}s)`));
			} else {
				console.error(c("31", `[orchestrate] Process exited with code ${exitCode}`));
				if (!verbose) {
					console.error("");
					console.error("Last 50 lines of output:");
					const allOutput = outputBuffer + stderrBuffer;
					console.error(allOutput.split("\n").slice(-50).join("\n"));
				}
			}
			process.exit(exitCode);
		}

		// Timeout
		const timeout = gameStarted ? FULL_TIMEOUT : EARLY_TIMEOUT;
		if (elapsed > timeout) {
			console.error(
				c(
					"31",
					`[orchestrate] Timeout (${timeout}s, game ${gameStarted ? "started" : "not started"})`,
				),
			);
			if (!verbose) {
				const allOutput = outputBuffer + stderrBuffer;
				console.error("Last 50 lines:");
				console.error(allOutput.split("\n").slice(-50).join("\n"));
			}
			await cleanup();
			process.exit(3);
		}

		await sleep(500);
	}
} catch (err) {
	console.error(c("31", `[orchestrate] Monitoring error: ${err}`));
	await cleanup();
	process.exit(1);
}
