/**
 * Run project checks in parallel. Auto-fixes formatting when safe.
 *
 * Usage: bun scripts/check.ts [--fix|-f] [--help|-h] [targets...]
 *
 * Targets scope checks to specific subsystems (comma-delimited):
 *   backend, frontend, mod (and aliases: b, f, m, rust, web, etc.)
 *   Omit targets to check everything.
 */

import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';
import { getCommand } from './lib/commands';
import { c, elapsed, isStderrTTY, parseFlags } from './lib/fmt';
import { type CollectResult, raceInOrder, run, runPiped, spawnCollect } from './lib/proc';
import { ensureFresh, newestMtime } from './lib/preflight';
import { type Subsystem, isAll, resolveTargets, targetLabel } from './lib/targets';

const { flags, passthrough } = parseFlags(
	process.argv.slice(2),
	{ fix: 'bool', help: 'bool' } as const,
	{ f: 'fix', h: 'help' },
	{ fix: false, help: false },
);

if (flags.help) {
	console.log(`Usage: bun scripts/check.ts [flags] [targets...]

Runs project checks in parallel. Auto-fixes formatting when safe.

Targets (comma-delimited, omit for all):
  backend (b, back, rust)     Backend: format, clippy, tests
  frontend (f, front, web)    Frontend: typecheck, lint, format, tests
  mod (m, minecraft)          Mod: format, lint, compile, tests

Flags:
  -f, --fix     Format code first, then verify
  -h, --help    Show this help message and exit

Examples:
  bun scripts/check.ts              Check everything
  bun scripts/check.ts backend      Check backend only
  bun scripts/check.ts b,f          Check backend + frontend
  bun scripts/check.ts mod --fix    Format mod, then verify`);
	process.exit(0);
}

const targets = resolveTargets(passthrough);
const has = (s: Subsystem) => targets.subsystems.has(s);
const targeted = !isAll(targets);

if (targeted) {
	process.stdout.write(c('1;36', `→ Checking: ${targetLabel(targets)}`) + '\n');
}

const fix = flags.fix;

if (fix) {
	process.stdout.write(c('1;36', '→ Fixing...') + '\n');
	if (has('frontend')) run(getCommand('frontend', 'format-apply').cmd);
	if (has('backend')) run(getCommand('backend', 'format-apply').cmd);
	if (has('mod')) {
		const def = getCommand('mod', 'format-apply');
		runPiped(def.cmd, { cwd: def.cwd });
	}
	process.stdout.write(c('1;36', '→ Verifying...') + '\n');
}

const rustSrcMtime = Math.max(
	newestMtime('backend/src', '**/*.rs'),
	...['backend/Cargo.toml', 'backend/Cargo.lock']
		.filter(existsSync)
		.map((f) => statSync(f).mtimeMs),
);

// Pre-flight: TS bindings (frontend depends on Rust types)
if (has('frontend')) {
	const BINDINGS_DIR = 'frontend/src/lib/bindings';
	ensureFresh({
		label: 'bindings',
		sourceMtime: rustSrcMtime,
		artifactDir: BINDINGS_DIR,
		artifactGlob: '**/*',
		reason: 'Rust sources changed',
		regenerate: () => {
			const tmpDir = mkdtempSync(join(tmpdir(), 'glint-bindings-'));
			try {
				for (const cmd of [
					{ cmd: ['cargo', 'test', '--lib', '--no-run', '--quiet'], opts: { cwd: 'backend' } },
					{
						cmd: ['cargo', 'test', '--lib', 'export_bindings', '--quiet'],
						opts: { cwd: 'backend', env: { TS_RS_EXPORT_DIR: tmpDir } },
					},
				]) {
					const result = runPiped(cmd.cmd, cmd.opts);
					if (result.exitCode !== 0) {
						if (result.stdout) process.stdout.write(result.stdout);
						if (result.stderr) process.stderr.write(result.stderr);
						process.exit(result.exitCode);
					}
				}

				if (!existsSync(BINDINGS_DIR)) {
					mkdirSync(BINDINGS_DIR, { recursive: true });
				}

				const newFiles = new Set(readdirSync(tmpDir).filter((f) => f.endsWith('.ts')));
				const oldFiles = new Set(
					readdirSync(BINDINGS_DIR).filter((f) => f.endsWith('.ts') && f !== 'index.ts'),
				);

				let changed = 0;
				for (const file of newFiles) {
					const newContent = readFileSync(join(tmpDir, file));
					const oldPath = join(BINDINGS_DIR, file);
					if (existsSync(oldPath)) {
						const oldContent = readFileSync(oldPath);
						if (Buffer.compare(newContent, oldContent) === 0) continue;
					}
					writeFileSync(oldPath, newContent);
					changed++;
				}

				for (const file of oldFiles) {
					if (!newFiles.has(file)) {
						rmSync(join(BINDINGS_DIR, file));
						changed++;
					}
				}

				run(['bun', 'scripts/bindings-barrel.ts']);

				const count = newFiles.size;
				const detail =
					changed > 0 ? `${count} types, ${changed} changed` : `${count} types, no changes`;
				return { detail };
			} finally {
				rmSync(tmpDir, { recursive: true, force: true });
			}
		},
	});
}

// Pre-flight: JSON schemas (mod tests use these for contract drift detection)
if (has('mod')) {
	ensureFresh({
		label: 'schemas',
		sourceMtime: rustSrcMtime,
		artifactDir: 'schemas',
		artifactGlob: '*.json',
		reason: 'Rust sources changed',
		regenerate: () => {
			run(['cargo', 'test', 'export_all_schemas', '--quiet'], { cwd: 'backend' });
			const count = readdirSync('schemas').filter((f) => f.endsWith('.json')).length;
			return { detail: `${count} types` };
		},
	});
}

// Pre-flight: SQLx query metadata (backend compile-time verification)
if (has('backend')) {
	const sqlxSrcMtime = Math.max(rustSrcMtime, newestMtime('backend/migrations', '*.sql'));
	ensureFresh({
		label: 'sqlx',
		sourceMtime: sqlxSrcMtime,
		artifactDir: 'backend/.sqlx',
		artifactGlob: '*.json',
		reason: 'sources changed',
		regenerate: () => {
			const result = runPiped(['cargo', 'sqlx', 'prepare'], { cwd: 'backend' });
			if (result.exitCode !== 0) {
				if (result.stderr) process.stderr.write(result.stderr);
				return { warning: 'sqlx prepare failed (is the database running?)' };
			}
			const count = existsSync('backend/.sqlx')
				? readdirSync('backend/.sqlx').filter((f) => f.endsWith('.json')).length
				: 0;
			return { detail: `${count} queries` };
		},
	});
}

// Pre-flight: GraphQL schema + gql-tada (frontend type checking)
if (has('frontend')) {
	const GQL_DIR = 'frontend/src/lib/graphql';

	ensureFresh({
		label: 'graphql schema',
		sourceMtime: rustSrcMtime,
		artifactDir: GQL_DIR,
		artifactGlob: 'schema.graphql',
		reason: 'Rust sources changed',
		regenerate: () => {
			const schemaPath = join(GQL_DIR, 'schema.graphql');
			const existing = existsSync(schemaPath) ? readFileSync(schemaPath) : null;

			const result = runPiped(['cargo', 'test', '--test', 'graphql_schema', '--quiet'], {
				cwd: 'backend',
			});
			if (result.exitCode !== 0) {
				if (result.stdout) process.stdout.write(result.stdout);
				if (result.stderr) process.stderr.write(result.stderr);
				process.exit(1);
			}

			const updated = readFileSync(schemaPath);
			const changed = !existing || Buffer.compare(existing, updated) !== 0;
			return changed ? { detail: 'updated' } : {};
		},
	});

	const schemaMtime = newestMtime(GQL_DIR, 'schema.graphql');
	ensureFresh({
		label: 'gql-tada',
		sourceMtime: schemaMtime,
		artifactDir: GQL_DIR,
		artifactGlob: 'graphql-env.d.ts',
		reason: 'schema changed',
		regenerate: () => {
			const envPath = join(GQL_DIR, 'graphql-env.d.ts');
			const existing = existsSync(envPath) ? readFileSync(envPath) : null;

			const result = runPiped(['bunx', 'gql-tada', 'generate', 'output'], {
				cwd: 'frontend',
			});
			if (result.exitCode !== 0) {
				if (result.stdout) process.stdout.write(result.stdout);
				if (result.stderr) process.stderr.write(result.stderr);
				process.exit(1);
			}

			const updated = readFileSync(envPath);
			const changed = !existing || Buffer.compare(existing, updated) !== 0;
			return changed ? { detail: 'updated' } : {};
		},
	});
}

interface Check {
	name: string;
	cmd: string[];
	cwd?: string;
	hint?: string;
	subsystem: 'frontend' | 'backend' | 'mod' | 'security';
}

/** Pick cmd/cwd fields from a registry entry for use in Check objects. */
const cmdOf = (sub: Subsystem, action: Parameters<typeof getCommand>[1]) => {
	const { cmd, cwd } = getCommand(sub, action);
	return cwd ? { cmd, cwd } : { cmd };
};

const allChecks: Check[] = [
	{ name: 'frontend-check', subsystem: 'frontend', ...cmdOf('frontend', 'type-check') },
	{ name: 'frontend-lint', subsystem: 'frontend', ...cmdOf('frontend', 'lint') },
	{
		name: 'frontend-format',
		subsystem: 'frontend',
		...cmdOf('frontend', 'format-check'),
		hint: "Run 'just format frontend' to fix.",
	},
	{
		name: 'backend-format',
		subsystem: 'backend',
		...cmdOf('backend', 'format-check'),
		hint: "Run 'just format backend' to fix.",
	},
	{ name: 'backend-lint', subsystem: 'backend', ...cmdOf('backend', 'lint') },
	{
		name: 'backend-test',
		subsystem: 'backend',
		cmd: [
			...getCommand('backend', 'test').cmd,
			'-E',
			'not test(export_bindings) and not test(export_graphql_sdl)',
		],
	},
	{
		name: 'mod-format',
		subsystem: 'mod',
		...cmdOf('mod', 'format-check'),
		hint: "Run 'just format mod' to fix.",
	},
	{ name: 'mod-lint', subsystem: 'mod', ...cmdOf('mod', 'lint') },
	{ name: 'mod-check', subsystem: 'mod', ...cmdOf('mod', 'compile') },
	{ name: 'mod-test', subsystem: 'mod', ...cmdOf('mod', 'test') },
	{ name: 'frontend-test', subsystem: 'frontend', ...cmdOf('frontend', 'test') },
	{
		name: 'backend-audit',
		subsystem: 'security',
		cmd: ['cargo', 'audit', '-f', 'backend/Cargo.lock'],
	},
	{
		name: 'frontend-audit',
		subsystem: 'security',
		cmd: ['bun', 'scripts/audit.ts'],
	},
];

// Filter checks: include matching subsystems + always include security
const checks = allChecks.filter(
	(ch) => ch.subsystem === 'security' || targets.subsystems.has(ch.subsystem as Subsystem)
);

const domains: Record<
	string,
	{
		peers: string[];
		format: () => ReturnType<typeof runPiped>;
		recheck: Check[];
	}
> = {
	'frontend-format': {
		peers: ['frontend-check', 'frontend-lint', 'frontend-test'],
		format: () => runPiped(getCommand('frontend', 'format-apply').cmd),
		recheck: [
			{ name: 'frontend-format', subsystem: 'frontend', ...cmdOf('frontend', 'format-check') },
			{ name: 'frontend-check', subsystem: 'frontend', ...cmdOf('frontend', 'type-check') },
		],
	},
	'backend-format': {
		peers: ['backend-lint', 'backend-test'],
		format: () => runPiped(getCommand('backend', 'format-apply').cmd),
		recheck: [
			{ name: 'backend-format', subsystem: 'backend', ...cmdOf('backend', 'format-check') },
			{ name: 'backend-lint', subsystem: 'backend', ...cmdOf('backend', 'lint') },
		],
	},
	'mod-format': {
		peers: ['mod-lint', 'mod-check', 'mod-test'],
		format: () => {
			const def = getCommand('mod', 'format-apply');
			return runPiped(def.cmd, { cwd: def.cwd });
		},
		recheck: [
			{ name: 'mod-format', subsystem: 'mod', ...cmdOf('mod', 'format-check') },
			{ name: 'mod-check', subsystem: 'mod', ...cmdOf('mod', 'compile') },
		],
	},
};

// Filter domains to only include targeted subsystems
const activeDomains = Object.fromEntries(
	Object.entries(domains).filter(([name]) => {
		const subsystem = name.replace('-format', '') as Subsystem;
		return targets.subsystems.has(subsystem);
	})
);

const start = Date.now();
const remaining = new Set(checks.map((ch) => ch.name));

const promises = checks.map(async (check) => ({
	...check,
	...(await spawnCollect(check.cmd, start, { cwd: check.cwd }))
}));

const interval = isStderrTTY
	? setInterval(() => {
			const cols = process.stderr.columns || 80;
			const line = `${elapsed(start)}s [${Array.from(remaining).join(', ')}]`;
			process.stderr.write(`\r\x1b[K${line.length > cols ? line.slice(0, cols - 1) + '…' : line}`);
		}, 100)
	: null;

const results: Record<string, Check & CollectResult> = {};

await raceInOrder(promises, checks, (r) => {
	results[r.name] = r;
	remaining.delete(r.name);
	if (isStderrTTY) process.stderr.write('\r\x1b[K');

	const subsystemLabel = c('2', `[${r.subsystem}]`);
	if (r.exitCode !== 0) {
		process.stdout.write(c('31', `✗ ${r.name}`) + ` ${subsystemLabel} (${r.elapsed}s)\n`);
		if (r.hint) {
			process.stdout.write(c('2', `  ${r.hint}`) + '\n');
		} else {
			if (r.stdout) process.stdout.write(r.stdout);
			if (r.stderr) process.stderr.write(r.stderr);
		}
	} else {
		process.stdout.write(c('32', `✓ ${r.name}`) + ` ${subsystemLabel} (${r.elapsed}s)\n`);
	}
});

if (interval) clearInterval(interval);
if (isStderrTTY) process.stderr.write('\r\x1b[K');

const autoFixedDomains = new Set<string>();

for (const [fmtName, domain] of Object.entries(activeDomains)) {
	const fmtResult = results[fmtName];
	if (!fmtResult || fmtResult.exitCode === 0) continue;
	// Only auto-fix if peers that were actually run all passed
	const runPeers = domain.peers.filter((p) => results[p]);
	if (runPeers.length === 0) continue;
	if (!runPeers.every((p) => results[p]?.exitCode === 0)) continue;

	process.stdout.write(
		'\n' + c('1;36', `→ Auto-formatting ${fmtName} (peers passed, only formatting failed)...`) + '\n'
	);
	const fmtOut = domain.format();
	if (fmtOut.exitCode !== 0) {
		process.stdout.write(c('31', `  ✗ ${fmtName} formatter failed`) + '\n');
		if (fmtOut.stdout) process.stdout.write(fmtOut.stdout);
		if (fmtOut.stderr) process.stderr.write(fmtOut.stderr);
		continue;
	}

	const recheckStart = Date.now();
	const recheckPromises = domain.recheck.map(async (ch) => ({
		...ch,
		...(await spawnCollect(ch.cmd, recheckStart, { cwd: ch.cwd }))
	}));

	let recheckFailed = false;
	await raceInOrder(recheckPromises, domain.recheck, (r) => {
		if (r.exitCode !== 0) {
			recheckFailed = true;
			process.stdout.write(c('31', `  ✗ ${r.name}`) + ` (${r.elapsed}s)\n`);
			if (r.stdout) process.stdout.write(r.stdout);
			if (r.stderr) process.stderr.write(r.stderr);
		} else {
			process.stdout.write(c('32', `  ✓ ${r.name}`) + ` (${r.elapsed}s)\n`);
		}
	});

	if (!recheckFailed) {
		process.stdout.write(c('32', `  ✓ ${fmtName} auto-fix succeeded`) + '\n');
		autoFixedDomains.add(fmtName);
	} else {
		process.stdout.write(c('31', `  ✗ ${fmtName} auto-fix failed sanity check`) + '\n');
	}
}

const finalFailed = Object.entries(results).some(
	([name, r]) => r.exitCode !== 0 && !autoFixedDomains.has(name)
);

if (autoFixedDomains.size > 0 && !finalFailed) {
	process.stdout.write('\n' + c('1;32', '✓ All checks passed (formatting was auto-fixed)') + '\n');
}

process.exit(finalFailed ? 1 : 0);
