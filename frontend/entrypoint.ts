import { spawn, type Subprocess } from 'bun';

const PORT = process.env.PORT || '8080';
const BACKEND_PORT = process.env.GLINT_PORT || '3001';
const BACKEND_HOST = process.env.GLINT_HOST || '127.0.0.1';
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/health`;
const LOG_JSON = process.env.LOG_JSON === 'true';

type LogLevel = 'info' | 'warn' | 'error' | 'debug';

function log(level: LogLevel, message: string, fields?: Record<string, unknown>) {
	if (LOG_JSON) {
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

log('info', 'Starting Axum backend', { host: BACKEND_HOST, port: BACKEND_PORT });
const rustProc = spawn({
	cmd: ['/app/glint'],
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
	cmd: ['bun', 'build/index.js'],
	cwd: '/app/web',
	env: {
		...process.env,
		PORT,
		HOST: '0.0.0.0',
		ORIGIN: process.env.ORIGIN ?? `http://localhost:${PORT}`,
		PUBLIC_BACKEND_URL: `http://localhost:${BACKEND_PORT}`
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
