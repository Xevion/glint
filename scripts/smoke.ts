#!/usr/bin/env bun

import { spawn, type Subprocess } from 'bun';
import * as fs from 'fs/promises';
import { existsSync } from 'fs';
import { randomUUID } from 'crypto';

// Configuration
const PLATFORM = Bun.argv[2] || 'fabric';
const SESSION_ID = randomUUID();
const SESSION_FILE = `/tmp/glint-smoke-${SESSION_ID}.session`;

// Paths
const OPTIONS_PATH = `mod/${PLATFORM}/run/options.txt`;

// Timeout detection
const ASSETS_PATH = `mod/${PLATFORM}/run/.minecraft/assets`;
const hasAssets = existsSync(ASSETS_PATH);
const TIMEOUT = hasAssets ? 120 : 300;
// Shorter timeout for early crash detection (gradle startup + initial launch)
const EARLY_TIMEOUT = 60;

// Error patterns - split into fatal errors and warnings
const FATAL_MIXIN_ERRORS = [
	/critical injection/i,
	/@Shadow.*not located/i,
	/transformation.*failed/i,
	/mixin apply.*failed/i,
	/Target method.*not found/i,
	/Mixin.*crashed/i
];

const MIXIN_WARNINGS: RegExp[] = [
	// Add warning patterns here for non-fatal mixin issues that should be reported
];

const BUILD_ERROR_PATTERNS = [
	/BUILD FAILED/,
	/compilation failed/i,
	/error: cannot find symbol/i,
	/Execution failed for task/i
];

// Early crash patterns - process will likely die immediately after these
// NOTE: ClassNotFoundException/NoClassDefFoundError require "Caused by:" context
// because the mixin system logs benign warnings when probing optional classes:
//   [WARN] (FabricLoader/Mixin) Error loading class: ... (java.lang.ClassNotFoundException: ...)
const EARLY_CRASH_PATTERNS = [
	/Caused by:.*ClassNotFoundException/,
	/Caused by:.*NoClassDefFoundError/,
	/Exception in thread "main"/,
	/Error: Could not find or load main class/,
];

const SUCCESS_MARKERS = [
	/Setting user:/,
	/OpenGL:/,
	/Backend library:/,
	/Narrator library:/,
	/Loaded \d+ mods/
];

// Markers indicating game has started loading (but not yet passed)
const GAME_STARTED_MARKERS = [
	/Loaded \d+ mods/,
	/FabricLoader\//,
	/Loading Minecraft/,
	/Mixin Subsystem/,
	/NeoForge/,
	/Architectury/
];

// State
let gradleProc: Subprocess | null = null;
let gradlePid: number | null = null;
let cleanupDone = false;
let stderrBuffer = '';

// Utilities
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function isProcessAlive(pid: number): boolean {
	try {
		process.kill(pid, 0);
		return true;
	} catch {
		return false;
	}
}

function extractErrorContext(logContent: string, pattern: RegExp): string {
	const lines = logContent.split('\n');
	for (let i = 0; i < lines.length; i++) {
		if (pattern.test(lines[i])) {
			const start = Math.max(0, i - 5);
			const end = Math.min(lines.length, i + 6);
			return lines.slice(start, end).join('\n');
		}
	}
	return 'Error context not found';
}

// Stream monitoring state
let outputBuffer = '';
let testPassed = false;
let testFailed = false;
let failureReason = '';
let failureExitCode = 1;
let warningCount = 0;
const warnings: string[] = [];
let gameStarted = false; // True once game loading has begun

// Cleanup handler
async function cleanup(): Promise<void> {
	if (cleanupDone) return;
	cleanupDone = true;

	console.log('[cleanup] Cleaning up...');

	if (gradlePid && isProcessAlive(gradlePid)) {
		try {
			// Graceful shutdown
			process.kill(gradlePid, 'SIGTERM');
			await sleep(5000);

			// Force kill if still alive
			if (isProcessAlive(gradlePid)) {
				console.log('   Force killing process...');
				process.kill(gradlePid, 'SIGKILL');
			}

			// Kill process group
			try {
				process.kill(-gradlePid, 'SIGTERM');
			} catch {
				// Process group may not exist
			}
		} catch (e) {
			// Process already dead
		}
	}

	// Remove session file
	try {
		await fs.unlink(SESSION_FILE);
	} catch {
		// File may not exist
	}
}

// Register cleanup handlers
process.on('exit', () => {
	if (!cleanupDone) {
		cleanup();
	}
});

process.on('SIGINT', async () => {
	await cleanup();
	process.exit(130);
});

process.on('SIGTERM', async () => {
	await cleanup();
	process.exit(143);
});

// Pre-flight checks
async function createQuietOptions(): Promise<void> {
	// Only create if doesn't exist (don't overwrite user settings)
	if (existsSync(OPTIONS_PATH)) {
		return;
	}

	const quietOptions = `soundCategory_master:0.0
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

	await fs.mkdir(`mod/${PLATFORM}/run`, { recursive: true });
	await fs.writeFile(OPTIONS_PATH, quietOptions);
}

// Monitor output stream (stdout or stderr)
async function monitorOutputStream(
	stream: ReadableStream,
	isStderr: boolean = false
): Promise<void> {
	try {
		const reader = stream.getReader();
		const decoder = new TextDecoder();

		while (true) {
			const { done, value } = await reader.read();
			if (done) break;

			const data = decoder.decode(value, { stream: true });

			// Add to appropriate buffer
			if (isStderr) {
				stderrBuffer += data;
			} else {
				outputBuffer += data;
			}

			// Show in verbose mode
			if (process.env.VERBOSE) {
				if (isStderr) {
					process.stderr.write(data);
				} else {
					process.stdout.write(data);
				}
			}

			// Check for build errors (fail fast)
			for (const pattern of BUILD_ERROR_PATTERNS) {
				if (pattern.test(data)) {
					testFailed = true;
					failureReason = `Build failed:\n${data}`;
					failureExitCode = 1;
					return;
				}
			}

			// Check for fatal mixin errors (fail fast)
			for (const pattern of FATAL_MIXIN_ERRORS) {
				if (pattern.test(data)) {
					testFailed = true;
					failureReason = `Fatal mixin error:\n${extractErrorContext(outputBuffer + stderrBuffer, pattern)}`;
					failureExitCode = 2;
					return;
				}
			}

			// Check for early crash patterns (process death imminent)
			for (const pattern of EARLY_CRASH_PATTERNS) {
				if (pattern.test(data)) {
					testFailed = true;
					failureReason = `Early crash detected:\n${extractErrorContext(outputBuffer + stderrBuffer, pattern)}`;
					failureExitCode = 5;
					return;
				}
			}

			// Check for mixin warnings (track but don't fail)
			for (const pattern of MIXIN_WARNINGS) {
				if (pattern.test(data)) {
					warningCount++;
					const context = extractErrorContext(outputBuffer + stderrBuffer, pattern);
					if (!warnings.includes(context)) {
						warnings.push(context);
					}
					// Don't return - continue monitoring
				}
			}

			// Check for game started markers (extends timeout)
			if (!gameStarted) {
				for (const marker of GAME_STARTED_MARKERS) {
					if (marker.test(data)) {
						gameStarted = true;
						break;
					}
				}
			}

			// Check for success markers
			for (const marker of SUCCESS_MARKERS) {
				if (marker.test(data)) {
					testPassed = true;
					return;
				}
			}
		}
	} catch (err) {
		// Stream closed or error - this is normal on process exit
	}
}

// Process spawning
function spawnMinecraft(): Subprocess {
	const args = [`:${PLATFORM}:runClient`, '--no-daemon', '--args=--nogui --width=854 --height=480'];

	const proc = spawn(['./gradlew', ...args], {
		cwd: 'mod',
		stdio: ['ignore', 'pipe', 'pipe'],
		env: {
			...process.env,
			GLINT_SESSION: SESSION_ID
		}
	});

	// Monitor both stdout and stderr for game output
	if (proc.stdout) {
		monitorOutputStream(proc.stdout, false);
	}

	if (proc.stderr) {
		monitorOutputStream(proc.stderr, true);
	}

	return proc;
}

// Wait for test completion by monitoring stream state
async function waitForCompletion(): Promise<void> {
	console.log('[wait] Waiting for game to start...');
	console.log('[wait] Monitoring output streams...');
	console.log(
		`[wait] Timeout: ${EARLY_TIMEOUT}s (early), ${TIMEOUT}s (${hasAssets ? 'assets cached' : 'first run'})`
	);

	const startTime = Date.now();

	try {
		while (true) {
			const elapsed = Math.floor((Date.now() - startTime) / 1000);

			// Check if test passed
			if (testPassed) {
				// Wait a moment for any late errors
				await sleep(2000);

				if (testFailed) {
					console.error('[FAIL] Late error detected!');
					console.error('');
					console.error(failureReason);
					console.error('');
					await cleanup();
					process.exit(failureExitCode);
				}

				console.log(`[PASS] Smoke test passed! (elapsed: ${elapsed}s)`);

				// Show warnings if any were detected
				if (warningCount > 0) {
					console.warn(`[WARN] ${warningCount} warning(s) detected`);
					if (process.env.VERBOSE) {
						console.warn('\nWarning details:');
						warnings.forEach((warning, idx) => {
							console.warn(`\nWarning ${idx + 1}:`);
							console.warn(warning);
						});
					} else {
						console.warn('   Run with VERBOSE=1 to see details');
					}
				}

				await cleanup();
				process.exit(0);
			}

			// Check if test failed
			if (testFailed) {
				console.error('[FAIL] Test failed!');
				console.error('');
				console.error(failureReason);
				console.error('');
				if (!process.env.VERBOSE) {
					console.error('[hint] Run with VERBOSE=1 for full output:');
					console.error('   VERBOSE=1 just smoke');
					console.error('');
					console.error('[hint] Or check the log file for errors:');
					console.error(
						`   grep -i "error\\|exception\\|fatal" mod/${PLATFORM}/run/logs/latest.log | tail -50`
					);
					console.error('');
				}
				await cleanup();
				process.exit(failureExitCode);
			}

			// Check timeout (use shorter timeout if game hasn't started loading)
			const currentTimeout = gameStarted ? TIMEOUT : EARLY_TIMEOUT;
			if (elapsed > currentTimeout) {
				console.error(
					`[FAIL] Timeout exceeded (${currentTimeout}s, game ${gameStarted ? 'started' : 'not started'})!`
				);
				console.error('');
				if (process.env.VERBOSE) {
					console.error('Full output:');
					const allOutput = outputBuffer + stderrBuffer;
					console.error(allOutput);
				} else {
					console.error('Last 100 lines of output:');
					const allOutput = outputBuffer + stderrBuffer;
					const lines = allOutput.split('\n');
					console.error(lines.slice(-100).join('\n'));
					console.error('');
					console.error('[hint] Run with VERBOSE=1 for full output:');
					console.error('   VERBOSE=1 just smoke');
					console.error('');
					console.error('[hint] Or check the log file for errors:');
					console.error(
						`   grep -i "error\\|exception\\|fatal" mod/${PLATFORM}/run/logs/latest.log | tail -50`
					);
				}
				await cleanup();
				process.exit(3);
			}

			// Check if process died without passing or failing
			if (gradlePid && !isProcessAlive(gradlePid)) {
				console.error('[FAIL] Process died unexpectedly!');
				console.error('');
				if (process.env.VERBOSE) {
					if (stderrBuffer) {
						console.error('Full stderr output:');
						console.error(stderrBuffer);
					}
					console.error('Full stdout output:');
					console.error(outputBuffer);
				} else {
					if (stderrBuffer) {
						console.error('Last 100 lines of stderr:');
						console.error(stderrBuffer.split('\n').slice(-100).join('\n'));
					} else {
						console.error('Last 100 lines of output:');
						const allOutput = outputBuffer + stderrBuffer;
						const lines = allOutput.split('\n');
						console.error(lines.slice(-100).join('\n'));
					}
					console.error('');
					console.error('[hint] Run with VERBOSE=1 for full output:');
					console.error('   VERBOSE=1 just smoke');
					console.error('');
					console.error('[hint] Or check the log file for errors:');
					console.error(
						`   grep -i "error\\|exception\\|fatal" mod/${PLATFORM}/run/logs/latest.log | tail -50`
					);
				}
				await cleanup();
				process.exit(4);
			}

			await sleep(500);
		}
	} catch (err) {
		console.error('[FAIL] Error during monitoring:', err);
		await cleanup();
		process.exit(4);
	}
}

// Main
async function main(): Promise<void> {
	try {
		console.log(`[start] Starting Minecraft ${PLATFORM} client...`);
		console.log(`   Session: ${SESSION_ID}`);

		// Write session file
		await fs.writeFile(SESSION_FILE, SESSION_ID);

		// Pre-flight setup (still create options.txt for quiet mode)
		await fs.mkdir(`mod/${PLATFORM}/run`, { recursive: true });
		await createQuietOptions();

		// Launch Minecraft
		gradleProc = spawnMinecraft();
		gradlePid = gradleProc.pid!;

		// Wait for completion (monitors stdout/stderr)
		await waitForCompletion();
	} catch (err) {
		console.error('[FAIL] Error in main:', err);
		await cleanup();
		process.exit(1);
	}
}

main().catch(async (err) => {
	console.error('[FAIL] Unexpected error:', err.message);
	if (err.stack) {
		console.error(err.stack);
	}
	await cleanup();
	process.exit(1);
});
