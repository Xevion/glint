import { spawn, type Subprocess } from 'bun';
import { existsSync } from 'fs';

const PORT = process.env.PORT || '8080';
const BACKEND_PORT = process.env.GLINT_PORT || '3001';
const BACKEND_HOST = process.env.GLINT_HOST || '127.0.0.1';
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/health`;

// Start Axum backend first (SvelteKit SSR needs it for load functions)
console.log(`Starting Axum backend on ${BACKEND_HOST}:${BACKEND_PORT}...`);
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
		console.error('ERROR: Axum backend failed to become healthy within 15s');
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
console.log('Axum backend is healthy');

// Start SvelteKit SSR (public-facing)
console.log(`Starting SvelteKit SSR on 0.0.0.0:${PORT}...`);
const bunProc = spawn({
	cmd: ['bun', 'build/index.js'],
	cwd: '/app/web',
	env: {
		...process.env,
		PORT,
		HOST: '0.0.0.0',
		PUBLIC_BACKEND_URL: `http://localhost:${BACKEND_PORT}`
	},
	stdout: 'inherit',
	stderr: 'inherit'
});

// Monitor both processes — exit if either dies
async function monitor(name: string, proc: Subprocess) {
	const exitCode = await proc.exited;
	console.error(`${name} exited with code ${exitCode}`);
	return { name, exitCode };
}

const result = await Promise.race([monitor('Axum', rustProc), monitor('SvelteKit', bunProc)]);

// Kill the other process
console.error(`${result.name} died, shutting down...`);
rustProc.kill();
bunProc.kill();
process.exit(result.exitCode || 1);
