#!/usr/bin/env bun
/**
 * Glint headless capture container entrypoint.
 *
 * Orchestrates: Xvfb → GPU detection → volume setup → Minecraft launch → window focus → cleanup.
 */

import { existsSync, readdirSync, symlinkSync, unlinkSync } from "node:fs";
import { mkdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import type { Subprocess } from "bun";

// ── Configuration ────────────────────────────────────────────────────────────

const MC_DIR = process.env.MC_DIR ?? "/minecraft";
const DISPLAY_NUM = process.env.DISPLAY_NUM ?? "99";
const RESOLUTION = process.env.RESOLUTION ?? "1920x1080x24";
const JAVA_MEM = process.env.JAVA_MEM ?? "4G";
const RECORD = process.env.RECORD === "true";

// ── Helpers ──────────────────────────────────────────────────────────────────

/** Run a command synchronously, returning stdout as trimmed text. Returns null on failure. */
function exec(cmd: string[], opts?: { env?: Record<string, string | undefined> }): string | null {
	const result = Bun.spawnSync(cmd, {
		stdout: "pipe",
		stderr: "pipe",
		env: { ...process.env, ...opts?.env },
	});
	return result.success ? result.stdout.toString().trim() : null;
}

/** Check if a command exists on PATH. */
function hasCommand(name: string): boolean {
	return Bun.spawnSync(["which", name], { stdout: "ignore", stderr: "ignore" }).success;
}

/** Sleep for the given number of milliseconds. */
const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

/** Read a metadata text file, returning its trimmed contents or a fallback. */
async function readMeta(name: string, fallback = ""): Promise<string> {
	const path = join(MC_DIR, name);
	if (!existsSync(path)) return fallback;
	return (await readFile(path, "utf-8")).trim();
}

/** Create a symlink, replacing any existing one. */
function forceSymlink(target: string, link: string): void {
	try { unlinkSync(link); } catch {}
	symlinkSync(target, link);
}

/** Count entries in a directory, returning 0 if it doesn't exist. */
function countDir(dir: string): number {
	try { return readdirSync(dir).length; } catch { return 0; }
}

// ── Process tracking ─────────────────────────────────────────────────────────

const managed: Subprocess[] = [];

function cleanup(): void {
	for (const proc of managed) {
		try { proc.kill("SIGTERM"); } catch {}
	}
}

process.on("SIGINT", () => { cleanup(); process.exit(130); });
process.on("SIGTERM", () => { cleanup(); process.exit(143); });

// ── Runtime environment ──────────────────────────────────────────────────────
// NVIDIA Container Toolkit mounts libraries at runtime that aren't in the build-time cache.
exec(["ldconfig"]);

// XDG_RUNTIME_DIR is required by various libraries (PulseAudio, Wayland probes, etc.)
if (!process.env.XDG_RUNTIME_DIR) {
	const xdgDir = "/tmp/xdg-runtime";
	Bun.spawnSync(["mkdir", "-p", xdgDir]);
	process.env.XDG_RUNTIME_DIR = xdgDir;
}

// Tell libraries we're running an X11 session (prevents Wayland detection attempts)
process.env.XDG_SESSION_TYPE = "x11";

// ── Banner ───────────────────────────────────────────────────────────────────

console.log("=== Glint Capture Container ===");
console.log(`  Display:     :${DISPLAY_NUM} (${RESOLUTION})`);
console.log(`  Java memory: ${JAVA_MEM}`);
console.log(`  Autonomous:  ${process.env.GLINT_AUTONOMOUS ?? "false"}`);
console.log(`  API URL:     ${process.env.GLINT_API_URL ?? "<not set>"}`);

// ── Start Xvfb ──────────────────────────────────────────────────────────────

console.log(`Starting Xvfb on :${DISPLAY_NUM}...`);

const xvfb = Bun.spawn(
	[
		"Xvfb", `:${DISPLAY_NUM}`, "-screen", "0", RESOLUTION, "-ac",
		"+extension", "GLX", "+extension", "RANDR", "+iglx", "+render",
		"-nolisten", "tcp", "-noreset", "-shmem",
	],
	{ stdout: "ignore", stderr: "ignore" },
);
managed.push(xvfb);
await sleep(2000);

if (xvfb.exitCode !== null) {
	console.error("ERROR: Xvfb failed to start");
	process.exit(1);
}

// Sync DISPLAY with DISPLAY_NUM (Dockerfile sets DISPLAY=:99 but user may override DISPLAY_NUM)
process.env.DISPLAY = `:${DISPLAY_NUM}`;

// ── GPU detection ────────────────────────────────────────────────────────────

let useVglrun = false;

const gpuName =
	hasCommand("nvidia-smi")
		? exec(["nvidia-smi", "--query-gpu=name", "--format=csv,noheader"])
		: null;

const hasNvidia = !!gpuName;
if (hasNvidia) console.log(`  NVIDIA GPU: ${gpuName}`);

if (hasNvidia && hasCommand("vglrun")) {
	// Remove Mesa EGL ICD so VirtualGL only finds NVIDIA
	try { unlinkSync("/usr/share/glvnd/egl_vendor.d/50_mesa.json"); } catch {}
	process.env.VGL_DISPLAY = "egl";

	const renderer = exec(["vglrun", "glxinfo"], { env: { VGL_DISPLAY: "egl" } });
	const gpuRenderer = renderer?.match(/OpenGL renderer.*:\s*(.*)/)?.[1] ?? "";

	if (gpuRenderer && /nvidia|geforce|gtx|rtx|quadro|tesla/i.test(gpuRenderer)) {
		console.log(`  GPU renderer: ${gpuRenderer}`);
		useVglrun = true;
	} else {
		console.warn(`  WARNING: VirtualGL EGL did not find NVIDIA GPU (got: ${gpuRenderer || "<empty>"})`);
		console.warn("  Check that libnvidia-gpucomp.so is mounted (nvidia-container-toolkit issue)");
		console.warn("  Falling back to software rendering");
	}
} else {
	if (!hasNvidia) console.warn("  WARNING: No NVIDIA GPU detected");
	if (!hasCommand("vglrun")) console.warn("  WARNING: vglrun not found");
	console.warn("  Falling back to software rendering (llvmpipe)");
}

// ── Symlink mounted volumes ──────────────────────────────────────────────────

if (existsSync("/shaderpacks")) {
	forceSymlink("/shaderpacks", join(MC_DIR, "shaderpacks"));
	console.log(`  Shader packs: ${countDir("/shaderpacks")} found`);
} else {
	console.log("  Shader packs: 0 found");
}

if (existsSync("/worlds")) {
	forceSymlink("/worlds", join(MC_DIR, "saves"));
	console.log(`  Worlds: ${countDir("/worlds")} found`);
} else {
	console.log("  Worlds: 0 found");
}

await mkdir("/output", { recursive: true });

// ── Copy default options.txt if missing ──────────────────────────────────────

const optionsPath = join(MC_DIR, "options.txt");
if (!existsSync(optionsPath)) {
	const src = Bun.file("/docker/options.txt");
	if (await src.exists()) {
		await Bun.write(optionsPath, src);
		console.log("  Installed default options.txt");
	}
}

// ── Read launch metadata from Stage 2 ────────────────────────────────────────

const [versionId, assetIndex, classpath, nativesDir] = await Promise.all([
	readMeta("version-id.txt"),
	readMeta("asset-index.txt"),
	readMeta("classpath.txt"),
	readMeta("natives-dir.txt", join(MC_DIR, "natives")),
]);

if (!classpath) {
	console.error(`ERROR: No classpath found at ${MC_DIR}/classpath.txt`);
	console.error("  Runtime assembly (Stage 2) may have failed.");
	process.exit(1);
}

if (!versionId) {
	console.error(`ERROR: No version ID found at ${MC_DIR}/version-id.txt`);
	process.exit(1);
}

const classpathCount = classpath.split(":").length;

console.log("=== Launching Minecraft ===");
console.log(`  Version:  ${versionId}`);
console.log(`  Assets:   ${assetIndex}`);
console.log(`  Natives:  ${nativesDir}`);
console.log(`  Classpath entries: ${classpathCount}`);

// ── Build and launch Java command ────────────────────────────────────────────

const javaArgs = [
	`-Xmx${JAVA_MEM}`,
	`-Xms${JAVA_MEM}`,
	"-XX:+UseG1GC",
	"-XX:+UnlockExperimentalVMOptions",
	"-XX:G1NewSizePercent=20",
	"-XX:G1ReservePercent=20",
	`-Djava.library.path=${nativesDir}`,
	"-Dglint.output.dir=/output",
	"-cp", classpath,
	"net.fabricmc.loader.impl.launch.knot.KnotClient",
	"--gameDir", MC_DIR,
	"--assetsDir", join(MC_DIR, "assets"),
	"--assetIndex", assetIndex,
	"--version", versionId,
	"--width", "1920",
	"--height", "1080",
	"--accessToken", "0",
];

const mcCmd = useVglrun ? ["vglrun", "java", ...javaArgs] : ["java", ...javaArgs];
const mc = Bun.spawn(mcCmd, { stdout: "inherit", stderr: "inherit", env: process.env });
managed.push(mc);

// ── Wait for Minecraft window and force X11 focus ────────────────────────────
// In Xvfb without a window manager, the window never gets FocusIn events.
// Minecraft's isWindowActive() returns false, which throttles chunk loading.

console.log("  Waiting for Minecraft window...");

const FOCUS_TIMEOUT = 60;
let focused = false;

for (let elapsed = 0; elapsed < FOCUS_TIMEOUT; elapsed++) {
	if (mc.exitCode !== null) {
		console.error("  ERROR: Minecraft exited before window appeared");
		process.exit(await mc.exited);
	}

	const windowId = exec(["xdotool", "search", "--name", "Minecraft"]);
	if (windowId) {
		const id = windowId.split("\n")[0];
		exec(["xdotool", "windowactivate", "--sync", id]);
		exec(["xdotool", "windowfocus", "--sync", id]);
		console.log(`  Window focused (id: ${id})`);
		focused = true;
		break;
	}

	await sleep(1000);
}

if (!focused) {
	console.warn(`  WARNING: Timed out waiting for Minecraft window (${FOCUS_TIMEOUT}s)`);
	console.warn("  Continuing without focus — chunks may not fully load");
}

// ── Optional screen recording ────────────────────────────────────────────────

let ffmpeg: Subprocess | null = null;
if (RECORD) {
	const [width, height] = RESOLUTION.split("x");
	const timestamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
	const recordPath = process.env.RECORD_PATH ?? `/output/capture-${timestamp}.mp4`;

	console.log(`  Recording to: ${recordPath}`);

	ffmpeg = Bun.spawn(
		[
			"ffmpeg", "-y", "-f", "x11grab",
			"-video_size", `${width}x${height}`,
			"-framerate", "30", "-i", `:${DISPLAY_NUM}`,
			"-c:v", "libx264", "-preset", "ultrafast", "-crf", "23", "-pix_fmt", "yuv420p",
			recordPath,
		],
		{ stdin: "ignore", stdout: "ignore", stderr: "ignore" },
	);
	managed.push(ffmpeg);
	console.log(`  Recording started (pid: ${ffmpeg.pid})`);
}

// ── Wait for Minecraft to exit ───────────────────────────────────────────────

const mcExit = await mc.exited;

if (ffmpeg && ffmpeg.exitCode === null) {
	ffmpeg.kill("SIGINT");
	await ffmpeg.exited;
	console.log("  Recording saved");
}

xvfb.kill("SIGTERM");

process.exit(mcExit);
