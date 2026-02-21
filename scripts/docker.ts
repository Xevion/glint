/**
 * Docker operations for Glint services.
 *
 * Usage: bun scripts/docker.ts [build|run] <web|capture>
 *
 * Omitting the action (build/run) does both: build then run.
 *
 * Examples:
 *   bun scripts/docker.ts build web       Build web server image
 *   bun scripts/docker.ts run capture     Run capture container
 *   bun scripts/docker.ts web             Build + run web server
 *   bun scripts/docker.ts capture         Build + run capture
 */

import { existsSync, readFileSync } from 'fs';
import { run, runPiped } from './lib/proc';

const argv = process.argv.slice(2);

if (argv[0] === '--help' || argv[0] === '-h' || argv.length === 0) {
	console.log(`Usage: bun scripts/docker.ts [build|run] <web|capture> [flags...]

Docker operations for Glint services.

Services:
  web        Web server (frontend + backend)
  capture    Headless Minecraft capture (requires NVIDIA GPU)

Actions:
  build      Build the Docker image
  run        Run the container
  (omit)     Build then run

Extra flags are passed through to docker build/run.

Examples:
  bun scripts/docker.ts build web
  bun scripts/docker.ts run capture
  bun scripts/docker.ts web             # build + run
  bun scripts/docker.ts capture         # build + run`);
	process.exit(argv.length === 0 ? 1 : 0);
}

type Action = 'build' | 'run';
type Service = 'web' | 'capture';

const ACTIONS = new Set<string>(['build', 'run']);
const SERVICES = new Set<string>(['web', 'capture']);

let action: Action | null = null;
let service: Service | null = null;
let extraFlags: string[] = [];

if (ACTIONS.has(argv[0])) {
	action = argv[0] as Action;
	if (!argv[1] || !SERVICES.has(argv[1])) {
		console.error(`Expected service (web or capture), got: ${argv[1] ?? '(nothing)'}`);
		process.exit(1);
	}
	service = argv[1] as Service;
	extraFlags = argv.slice(2);
} else if (SERVICES.has(argv[0])) {
	service = argv[0] as Service;
	extraFlags = argv.slice(1);
} else {
	console.error(`Unknown argument: ${argv[0]}\nExpected: [build|run] <web|capture>`);
	process.exit(1);
}

function buildWeb(flags: string[]): void {
	run(['docker', 'build', '-t', 'glint-web:latest', ...flags, '.']);
}

function runWeb(flags: string[]): void {
	if (!existsSync('backend/.env')) {
		console.error('ERROR: backend/.env not found');
		process.exit(1);
	}
	run([
		'docker', 'run', '--rm', '-it',
		'--network', 'host',
		'--env-file', 'backend/.env',
		...flags,
		'glint-web:latest',
	]);
}

function findGpucompMount(): string[] {
	const result = runPiped([
		'find', '/usr/lib/x86_64-linux-gnu',
		'-name', 'libnvidia-gpucomp.so.*',
		'-not', '-name', '*.so',
	]);
	const path = result.stdout.trim().split('\n')[0];
	if (path) {
		return ['-v', `${path}:${path}:ro`];
	}
	return [];
}

function loadEnv(): Record<string, string> {
	const env: Record<string, string> = { ...process.env as Record<string, string> };
	if (existsSync('.env')) {
		for (const line of readFileSync('.env', 'utf-8').split('\n')) {
			const trimmed = line.trim();
			if (!trimmed || trimmed.startsWith('#')) continue;
			const eq = trimmed.indexOf('=');
			if (eq === -1) continue;
			const key = trimmed.slice(0, eq);
			let val = trimmed.slice(eq + 1);
			// Strip surrounding quotes
			if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
				val = val.slice(1, -1);
			}
			env[key] = val;
		}
	}
	return env;
}

function buildCapture(flags: string[]): void {
	run(['docker', 'build', '-t', 'glint-capture:latest', '-f', 'docker/Dockerfile', ...flags, '.']);
}

function runCapture(flags: string[]): void {
	const env = loadEnv();

	if (!env.GLINT_API_TOKEN) {
		console.error('ERROR: GLINT_API_TOKEN must be set in .env');
		process.exit(1);
	}

	const gpucompMount = findGpucompMount();

	run([
		'docker', 'run', '--rm',
		'--gpus', 'all',
		'--device=/dev/dri:/dev/dri',
		'--add-host=host.docker.internal:host-gateway',
		'-e', 'NVIDIA_DRIVER_CAPABILITIES=all',
		'-e', `GLINT_AUTONOMOUS=${env.GLINT_AUTONOMOUS ?? 'true'}`,
		'-e', `GLINT_API_URL=${env.GLINT_API_URL ?? 'http://host.docker.internal:8080'}`,
		'-e', `GLINT_API_TOKEN=${env.GLINT_API_TOKEN}`,
		'-e', `RECORD=${env.RECORD ?? 'false'}`,
		'-v', 'mc-assets:/minecraft/assets',
		'-v', `${process.cwd()}/docker/output:/output`,
		...gpucompMount,
		...flags,
		'glint-capture:latest',
	]);
}

const actions: Action[] = action ? [action] : ['build', 'run'];

for (const act of actions) {
	if (service === 'web') {
		if (act === 'build') buildWeb(extraFlags);
		else runWeb(extraFlags);
	} else {
		if (act === 'build') buildCapture(extraFlags);
		else runCapture(extraFlags);
	}
}
