/**
 * Run project tests across subsystems.
 *
 * Usage: bun scripts/test.ts [targets...]
 *
 * Targets scope tests to specific subsystems (comma-delimited):
 *   backend, frontend, mod, e2e (and aliases: b, f, m, rust, web, etc.)
 *   Omit targets to run all unit tests in parallel.
 */

import { ProcessGroup, run } from './lib/proc';
import { isAll, resolveTargets } from './lib/targets';

const argv = process.argv.slice(2);

if (argv[0] === '--help' || argv[0] === '-h') {
	console.log(`Usage: bun scripts/test.ts [targets...]

Runs tests for one or more subsystems.

Targets (comma-delimited, omit for all):
  backend (b, back, rust)     Backend tests (cargo nextest)
  frontend (f, front, web)    Frontend unit tests (Vitest)
  mod (m, minecraft)          Mod tests (Gradle)
  e2e                         Frontend E2E tests (Playwright)

Examples:
  bun scripts/test.ts              All unit tests (parallel)
  bun scripts/test.ts backend      Backend only
  bun scripts/test.ts web,mod      Frontend + mod
  bun scripts/test.ts e2e          Playwright E2E tests`);
	process.exit(0);
}

const targets = resolveTargets(argv);
const all = isAll(targets) && !targets.e2e;

if (all) {
	const group = new ProcessGroup();
	group.spawn(['bun', 'run', '--cwd', 'frontend', 'test:unit']);
	group.spawn(['cargo', 'nextest', 'run', '--manifest-path', 'backend/Cargo.toml']);
	group.spawn(['./gradlew', 'test', '--quiet'], { cwd: 'mod' });
	const code = await group.waitForAll();
	process.exit(code);
}

const group = new ProcessGroup();
let spawned = 0;

if (targets.subsystems.has('frontend') && !targets.e2e) {
	group.spawn(['bun', 'run', '--cwd', 'frontend', 'test:unit']);
	spawned++;
}
if (targets.e2e) {
	group.spawn(['bun', 'run', '--cwd', 'frontend', 'test:e2e']);
	spawned++;
}
if (targets.subsystems.has('backend')) {
	group.spawn(['cargo', 'nextest', 'run', '--manifest-path', 'backend/Cargo.toml']);
	spawned++;
}
if (targets.subsystems.has('mod')) {
	group.spawn(['./gradlew', 'test', '--quiet'], { cwd: 'mod' });
	spawned++;
}

if (spawned === 0) {
	console.error('No tests to run for the given targets.');
	process.exit(1);
}

if (spawned === 1) {
	// Single target: let it inherit stdio directly for better output
	const code = await group.waitForAll();
	process.exit(code);
}

const code = await group.waitForAll();
process.exit(code);
