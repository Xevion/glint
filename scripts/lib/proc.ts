/**
 * Shared process spawning utilities for Glint project scripts.
 *
 * Provides utilities for synchronous/asynchronous process execution,
 * coordinated process groups, and parallel command execution with
 * result ordering.
 */

import { elapsed } from "./fmt";

/**
 * Result from collecting process output asynchronously.
 */
export interface CollectResult {
	/** Standard output captured from the process */
	stdout: string;
	/** Standard error captured from the process */
	stderr: string;
	/** Exit code (0 = success, non-zero = failure) */
	exitCode: number;
	/** Formatted elapsed time (e.g., "1.2s") */
	elapsed: string;
}

/**
 * Spawn a command synchronously with inherited stdio.
 *
 * The process will inherit stdin, stdout, and stderr, allowing
 * interactive commands and preserving color output. Exits the
 * parent process if the command fails.
 *
 * @param cmd - Command array [program, ...args]
 * @throws Exits process with child's exit code on failure
 *
 * @example
 * run(["cargo", "build", "--release"]);
 * run(["bun", "run", "test"]);
 */
export function run(cmd: string[]): void {
	const proc = Bun.spawnSync(cmd, { stdio: ["inherit", "inherit", "inherit"] });
	if (proc.exitCode !== 0) process.exit(proc.exitCode);
}

/**
 * Spawn a command synchronously with captured output.
 *
 * Unlike `run()`, this captures stdout/stderr instead of inheriting
 * stdio. Useful for parsing command output or checking for specific
 * text in the output.
 *
 * @param cmd - Command array [program, ...args]
 * @returns Object with exitCode, stdout, and stderr
 *
 * @example
 * const { exitCode, stdout } = runPiped(["git", "status", "--short"]);
 * if (exitCode === 0) {
 *   const modified = stdout.split("\n").filter(l => l.startsWith(" M"));
 * }
 */
export function runPiped(cmd: string[]): {
	exitCode: number;
	stdout: string;
	stderr: string;
} {
	const proc = Bun.spawnSync(cmd, { stdout: "pipe", stderr: "pipe" });
	return {
		exitCode: proc.exitCode,
		stdout: proc.stdout?.toString() ?? "",
		stderr: proc.stderr?.toString() ?? "",
	};
}

/**
 * Spawn a command asynchronously and collect output.
 *
 * Enables FORCE_COLOR=1 to preserve colored output in piped processes.
 * Catches spawn failures (e.g., command not found) and returns them as
 * CollectResult instead of throwing.
 *
 * @param cmd - Command array [program, ...args]
 * @param startTime - Timestamp from Date.now() for elapsed calculation
 * @returns Promise resolving to collected output and exit code
 *
 * @example
 * const start = Date.now();
 * const result = await spawnCollect(["cargo", "clippy"], start);
 * if (result.exitCode !== 0) {
 *   console.error(`clippy failed in ${result.elapsed}s`);
 * }
 */
export async function spawnCollect(
	cmd: string[],
	startTime: number,
): Promise<CollectResult> {
	try {
		const proc = Bun.spawn(cmd, {
			env: { ...process.env, FORCE_COLOR: "1" },
			stdout: "pipe",
			stderr: "pipe",
		});
		const [stdout, stderr] = await Promise.all([
			new Response(proc.stdout).text(),
			new Response(proc.stderr).text(),
		]);
		await proc.exited;
		return {
			stdout,
			stderr,
			exitCode: proc.exitCode ?? 1,
			elapsed: elapsed(startTime),
		};
	} catch (err) {
		return {
			stdout: "",
			stderr: String(err),
			exitCode: 1,
			elapsed: elapsed(startTime),
		};
	}
}

/**
 * Execute promises in parallel, yielding results in completion order.
 *
 * Unlike Promise.all(), this calls the callback as each promise completes,
 * allowing for progress reporting. Spawn failures are caught and converted
 * to error results using the fallback metadata.
 *
 * Useful for running multiple subsystem checks (frontend, backend, mod)
 * in parallel and reporting results as they complete.
 *
 * @param promises - Array of promises to execute
 * @param fallbacks - Metadata for each promise (used on failure)
 * @param onResult - Callback invoked as each result completes
 *
 * @example
 * const checks = [
 *   spawnCollect(["bun", "run", "typecheck"], Date.now()),
 *   spawnCollect(["cargo", "clippy"], Date.now()),
 * ];
 * await raceInOrder(
 *   checks,
 *   [{ name: "frontend" }, { name: "backend" }],
 *   (result) => console.log(`${result.name}: ${result.exitCode}`)
 * );
 */
export async function raceInOrder<T extends { name: string }>(
	promises: Promise<T & CollectResult>[],
	fallbacks: T[],
	onResult: (r: T & CollectResult) => void,
): Promise<void> {
	const tagged = promises.map((p, i) =>
		p
			.then((r) => ({ i, r }))
			.catch((err) => ({
				i,
				r: {
					...fallbacks[i],
					exitCode: 1,
					stdout: "",
					stderr: String(err),
					elapsed: "?",
				} as T & CollectResult,
			})),
	);

	for (let n = 0; n < promises.length; n++) {
		const { i, r } = await Promise.race(tagged);
		tagged[i] = new Promise(() => {}); // sentinel: never resolves
		onResult(r);
	}
}

/**
 * Managed process group with coordinated lifecycle and cleanup.
 *
 * Spawns multiple processes and ensures they are all killed when:
 * - Any process exits (via waitForFirst)
 * - Parent receives SIGINT/SIGTERM
 * - Explicit killAll() is called
 *
 * Essential for Glint's multi-subsystem development (frontend, backend, mod)
 * where all processes should start/stop together.
 *
 * @example
 * const group = new ProcessGroup();
 * group.spawn(["bun", "run", "--cwd", "frontend", "dev"]);
 * group.spawn(["cargo", "run", "--manifest-path", "backend/Cargo.toml"]);
 * const exitCode = await group.waitForFirst();
 * process.exit(exitCode);
 */
export class ProcessGroup {
	private procs: ReturnType<typeof Bun.spawn>[] = [];
	private cleanupRegistered = false;

	constructor() {
		// Register cleanup handlers to kill all processes on exit
		const cleanup = async () => {
			await this.killAll();
			process.exit(0);
		};
		process.on("SIGINT", cleanup);
		process.on("SIGTERM", cleanup);
		this.cleanupRegistered = true;
	}

	/**
	 * Spawn a new process in the group with inherited stdio.
	 *
	 * @param cmd - Command array [program, ...args]
	 * @param options - Optional spawn options (overrides defaults)
	 * @returns Bun spawn process handle
	 */
	spawn(
		cmd: string[],
		options?: { env?: Record<string, string>; cwd?: string },
	): ReturnType<typeof Bun.spawn> {
		const proc = Bun.spawn(cmd, {
			stdio: ["inherit", "inherit", "inherit"],
			env: { ...process.env, ...options?.env },
			cwd: options?.cwd,
		});
		this.procs.push(proc);
		return proc;
	}

	/**
	 * Kill all processes in the group and wait for them to exit.
	 *
	 * Sends SIGTERM to all processes and waits for clean shutdown.
	 */
	async killAll(): Promise<void> {
		for (const p of this.procs) p.kill();
		await Promise.all(this.procs.map((p) => p.exited));
	}

	/**
	 * Wait for any process to exit, then kill the rest.
	 *
	 * Returns the exit code of the first process to exit. Useful for
	 * development servers where any subsystem crash should stop all others.
	 *
	 * @returns Exit code of the first process to exit
	 */
	async waitForFirst(): Promise<number> {
		const results = this.procs.map((p, i) => p.exited.then((code) => ({ i, code })));
		const first = await Promise.race(results);
		await this.killAll();
		return first.code;
	}
}
