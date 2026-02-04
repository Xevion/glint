/**
 * Dev-agent orchestrator - builds mod, deploys JAR, runs agent.
 *
 * Usage: bun scripts/dev-agent.ts [dev-agent-flags] [agent-args...]
 *
 * Dev-agent flags (must come first):
 *   -s, --skip-build    Skip mod build (use existing JAR)
 *   -p, --platform X    Mod platform: fabric (default) or neoforge
 *   -v, --verbose       Verbose output
 *
 * All other args are passed to the agent binary.
 *
 * Examples:
 *   just dev-agent                    # Build mod, run agent in loop mode
 *   just dev-agent --once             # Build mod, run agent once
 *   just dev-agent -s --once          # Skip build, run once
 *   just dev-agent --dev-shader bsl --dev-scenes sunset,cave
 */

import { existsSync, statSync, readdirSync, copyFileSync, unlinkSync } from "fs";
import { join, basename } from "path";
import { c } from "./lib/fmt";
import { run } from "./lib/proc";

const MOD_DIR = "mod";
const MC_DIR = ".minecraft";
const MODS_DIR = join(MC_DIR, "mods");
const AGENT_MANIFEST = "agent/Cargo.toml";

// Parse args: dev-agent flags are consumed, everything else goes to agent
let skipBuild = false;
let platform = "fabric";
let verbose = false;
const agentArgs: string[] = [];

const argv = process.argv.slice(2);
let i = 0;
while (i < argv.length) {
	const arg = argv[i];

	// Dev-agent flags (consume these)
	if (arg === "-s" || arg === "--skip-build") {
		skipBuild = true;
		i++;
	} else if (arg === "-p" || arg === "--platform") {
		i++;
		platform = argv[i] ?? "fabric";
		i++;
	} else if (arg === "-v" || arg === "--verbose") {
		verbose = true;
		i++;
	} else {
		// Everything else goes to agent (including --)
		agentArgs.push(...argv.slice(i));
		break;
	}
}

// Validate platform
if (platform !== "fabric" && platform !== "neoforge") {
	console.error(c("31", `Invalid platform: ${platform} (must be 'fabric' or 'neoforge')`));
	process.exit(1);
}

// Check backend is running
async function checkBackend(): Promise<boolean> {
	try {
		const response = await fetch("http://localhost:8080/health", {
			method: "GET",
			signal: AbortSignal.timeout(2000),
		});
		return response.ok;
	} catch {
		return false;
	}
}

// Find the built mod JAR
function findModJar(): string | null {
	const buildLibs = join(MOD_DIR, platform, "build", "libs");
	if (!existsSync(buildLibs)) return null;

	const jars = readdirSync(buildLibs).filter(
		(f) => f.endsWith(".jar") && !f.includes("-dev-shadow") && !f.includes("-sources"),
	);

	if (jars.length === 0) return null;

	// Return the main JAR (not dev-shadow or sources)
	return join(buildLibs, jars[0]);
}

// Deploy mod JAR to .minecraft/mods
function deployMod(srcJar: string): void {
	const jarName = basename(srcJar);
	const destJar = join(MODS_DIR, jarName);

	// Remove old glint JARs (any version)
	const oldJars = readdirSync(MODS_DIR).filter(
		(f) => f.startsWith("glint-") && f.endsWith(".jar"),
	);
	for (const old of oldJars) {
		if (old !== jarName) {
			const oldPath = join(MODS_DIR, old);
			console.log(c("33", `  Removing old JAR: ${old}`));
			unlinkSync(oldPath);
		}
	}

	// Copy new JAR
	copyFileSync(srcJar, destJar);
	console.log(c("32", `  Deployed: ${jarName}`));
}

// Get newest mtime in a directory tree
function newestMtime(dir: string, extensions: string[]): number {
	let newest = 0;

	function walk(d: string): void {
		if (!existsSync(d)) return;
		for (const entry of readdirSync(d, { withFileTypes: true })) {
			const path = join(d, entry.name);
			if (entry.isDirectory()) {
				// Skip build directories
				if (entry.name === "build" || entry.name === ".gradle") continue;
				walk(path);
			} else if (extensions.some((ext) => entry.name.endsWith(ext))) {
				const stat = statSync(path);
				if (stat.mtimeMs > newest) newest = stat.mtimeMs;
			}
		}
	}

	walk(dir);
	return newest;
}

// Check if rebuild is needed (source newer than JAR)
function needsRebuild(jarPath: string): boolean {
	if (!existsSync(jarPath)) return true;

	const jarMtime = statSync(jarPath).mtimeMs;
	const sourceMtime = newestMtime(join(MOD_DIR, "common", "src"), [".kt", ".java"]);
	const platformMtime = newestMtime(join(MOD_DIR, platform, "src"), [".kt", ".java"]);

	return Math.max(sourceMtime, platformMtime) > jarMtime;
}

async function main(): Promise<void> {
	console.log(c("1;36", "=== Glint Dev Agent ===\n"));

	// 1. Check backend
	console.log(c("36", "→ Checking backend..."));
	const backendOk = await checkBackend();
	if (!backendOk) {
		console.error(c("31", "✗ Backend not running at http://localhost:8080"));
		console.error(c("33", "  Start it with: just dev -b"));
		process.exit(1);
	}
	console.log(c("32", "✓ Backend is running\n"));

	// 2. Build mod (unless skipped)
	let jarPath = findModJar();

	if (skipBuild) {
		if (!jarPath) {
			console.error(c("31", "✗ No mod JAR found and --skip-build specified"));
			console.error(c("33", `  Build manually: cd mod && ./gradlew :${platform}:build`));
			process.exit(1);
		}
		console.log(c("33", `→ Skipping mod build (using existing JAR)\n`));
	} else {
		// Check if rebuild is actually needed
		const existingJar = jarPath;
		const needsBuild = !existingJar || needsRebuild(existingJar);

		if (!needsBuild) {
			console.log(c("32", "→ Mod JAR is up-to-date, skipping build\n"));
		} else {
			console.log(c("36", `→ Building mod (${platform})...`));
			const buildStart = Date.now();

			// Run gradle from mod directory
			const proc = Bun.spawnSync(
				["./gradlew", `:${platform}:build`, "-x", "test", "--quiet"],
				{
					cwd: MOD_DIR,
					stdout: "pipe",
					stderr: "pipe",
				},
			);

			if (proc.exitCode !== 0) {
				console.error(c("31", "✗ Mod build failed"));
				const stderr = proc.stderr?.toString();
				if (stderr) console.error(stderr);
				process.exit(1);
			}

			const elapsed = ((Date.now() - buildStart) / 1000).toFixed(1);
			console.log(c("32", `✓ Mod built in ${elapsed}s\n`));

			// Refresh JAR path after build
			jarPath = findModJar();
		}
	}

	if (!jarPath) {
		console.error(c("31", "✗ No mod JAR found after build"));
		process.exit(1);
	}

	// 3. Deploy mod
	console.log(c("36", "→ Deploying mod..."));
	deployMod(jarPath);
	console.log();

	// 4. Run agent
	console.log(c("36", "→ Starting agent..."));
	const cargoArgs = ["cargo", "run", "--manifest-path", AGENT_MANIFEST, "--"];

	// Add verbose flag if requested
	if (verbose) {
		cargoArgs.push("-v");
	}

	// Add all passthrough args
	cargoArgs.push(...agentArgs);

	if (verbose) {
		console.log(c("2;37", `  Command: ${cargoArgs.join(" ")}`));
	}
	console.log();

	// Run agent (this will block until agent exits or is killed)
	run(cargoArgs);
}

main().catch((err) => {
	console.error(c("31", `Error: ${err.message}`));
	process.exit(1);
});
