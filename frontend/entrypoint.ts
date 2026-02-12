import { type Subprocess, spawn } from 'bun';

const PORT = process.env.PORT || '8080';
const BACKEND_PORT = process.env.GLINT_PORT || '3001';
const BACKEND_HOST = process.env.GLINT_HOST || '127.0.0.1';
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/health`;

// Logging defaults: JSON in production (entrypoint only runs in Docker).
// Both the Rust backend and SvelteKit frontend read LOG_JSON identically,
// so we normalize once here and propagate via env to both subprocesses.
const LOG_JSON = process.env.LOG_JSON ?? 'true';
const LOG_LEVEL = process.env.LOG_LEVEL; // undefined = let each subsystem pick its default

type LogLevel = 'info' | 'warn' | 'error' | 'debug';

function log(level: LogLevel, message: string, fields?: Record<string, unknown>) {
	if (LOG_JSON === 'true' || LOG_JSON === '1') {
		const entry = {
			timestamp: new Date().toISOString(),
			level,
			target: 'glint::entrypoint',
			message,
			...fields
		};
		const out = level === 'error' ? process.stderr : process.stdout;
		out.write(JSON.stringify(entry) + '\n');
	} else {
		const prefix = level === 'error' ? 'ERROR: ' : '';
		const suffix = fields
			? ` ${Object.entries(fields)
					.map(([k, v]) => `${k}=${v}`)
					.join(' ')}`
			: '';
		const out = level === 'error' ? console.error : console.log;
		out(`${prefix}${message}${suffix}`);
	}
}

// Build shared env for both subprocesses — ensures LOG_JSON and LOG_LEVEL
// are propagated even if the parent env didn't have them explicitly set.
const sharedEnv: Record<string, string | undefined> = {
	...process.env,
	LOG_JSON
};
if (LOG_LEVEL) {
	sharedEnv.LOG_LEVEL = LOG_LEVEL;
}

log('info', 'Starting Axum backend', { host: BACKEND_HOST, port: BACKEND_PORT });
const rustProc = spawn({
	cmd: ['/app/glint'],
	env: sharedEnv,
	stdout: 'inherit',
	stderr: 'inherit'
});

// Wait for backend to be healthy (15s timeout)
const startTime = Date.now();
let healthy = false;
while (!healthy) {
	if (Date.now() - startTime > 15_000) {
		log('error', 'Axum backend failed to become healthy within 15s');
		rustProc.kill();
		process.exit(1);
	}

	try {
		const response = await fetch(HEALTH_URL);
		if (response.ok) {
			healthy = true;
		}
	} catch {
		// Backend not ready yet
	}

	if (!healthy) {
		await Bun.sleep(250);
	}
}
log('info', 'Axum backend is healthy');

log('info', 'Starting SvelteKit SSR', { host: '0.0.0.0', port: PORT });
const bunProc = spawn({
	cmd: ['bun', '--preload', '/app/web/console-logger.js', 'build/index.js'],
	cwd: '/app/web',
	env: {
		...sharedEnv,
		PORT,
		HOST: '0.0.0.0',
		ORIGIN: process.env.ORIGIN ?? `http://localhost:${PORT}`,
		BACKEND_URL: `http://localhost:${BACKEND_PORT}`
	},
	stdout: 'inherit',
	stderr: 'inherit'
});

// Monitor both processes — exit if either dies
async function monitor(name: string, proc: Subprocess) {
	const exitCode = await proc.exited;
	log('error', `${name} exited`, { exit_code: exitCode });
	return { name, exitCode };
}

const result = await Promise.race([monitor('Axum', rustProc), monitor('SvelteKit', bunProc)]);

log('error', 'Shutting down', { trigger: result.name });
rustProc.kill();
bunProc.kill();
process.exit(result.exitCode || 1);
