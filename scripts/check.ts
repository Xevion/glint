/**
 * Run all project checks in parallel. Auto-fixes formatting when safe.
 *
 * Usage: bun scripts/check.ts [--fix|-f] [--help|-h]
 */

import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';
import { c, elapsed, isStderrTTY, parseFlags } from './lib/fmt';
import { type CollectResult, raceInOrder, run, runPiped, spawnCollect } from './lib/proc';

/** Scan files matching a glob pattern and return the newest mtime (ms). */
function newestMtime(dir: string, pattern: string): number {
	let newest = 0;
	for (const file of new Bun.Glob(pattern).scanSync(dir)) {
		const mt = statSync(`${dir}/${file}`).mtimeMs;
		if (mt > newest) newest = mt;
	}
	return newest;
}

const { flags } = parseFlags(
	process.argv.slice(2),
	{ fix: 'bool', help: 'bool' } as const,
	{ f: 'fix', h: 'help' },
	{ fix: false, help: false },
);

if (flags.help) {
	console.log(`Usage: bun scripts/check.ts [flags]

Runs all project checks in parallel. Auto-fixes formatting when safe.

Flags:
  -f, --fix     Format code first, then verify
  -h, --help    Show this help message and exit`);
	process.exit(0);
}

const fix = flags.fix;

if (fix) {
	console.log(c('1;36', '→ Fixing...'));
	run(['bun', 'run', '--cwd', 'frontend', 'format']);
	run(['cargo', 'fmt', '--manifest-path', 'backend/Cargo.toml']);
	runPiped(['./gradlew', 'spotlessApply', 'ktlintFormat', '--quiet'], { cwd: 'mod' });
	console.log(c('1;36', '→ Verifying...'));
}

const rustSrcMtime = Math.max(
	newestMtime('backend/src', '**/*.rs'),
	...['backend/Cargo.toml', 'backend/Cargo.lock']
		.filter(existsSync)
		.map((f) => statSync(f).mtimeMs),
);

{
	const BINDINGS_DIR = 'frontend/src/lib/bindings';

	const newestBindingMtime = existsSync(BINDINGS_DIR)
		? newestMtime(BINDINGS_DIR, '**/*')
		: 0;

	const stale = newestBindingMtime === 0 || rustSrcMtime > newestBindingMtime;
	if (stale) {
		const t = Date.now();
		process.stdout.write(
			c('1;36', '→ Regenerating TypeScript bindings (Rust sources changed)...') + '\n'
		);

		// Generate into a temp directory to avoid triggering HMR for every file
		const tmpDir = mkdtempSync(join(tmpdir(), 'glint-bindings-'));
		try {
			for (const cmd of [
				{ cmd: ['cargo', 'test', '--no-run', '--quiet'], opts: { cwd: 'backend' } },
				{
					cmd: ['cargo', 'test', 'export_bindings', '--quiet'],
					opts: { cwd: 'backend', env: { TS_RS_EXPORT_DIR: tmpDir } }
				}
			]) {
				const result = runPiped(cmd.cmd, cmd.opts);
				if (result.exitCode !== 0) {
					if (result.stdout) process.stdout.write(result.stdout);
					if (result.stderr) process.stderr.write(result.stderr);
					process.exit(result.exitCode);
				}
			}

			// Diff-sync: only write files whose content actually changed
			if (!existsSync(BINDINGS_DIR)) {
				mkdirSync(BINDINGS_DIR, { recursive: true });
			}

			const newFiles = new Set(readdirSync(tmpDir).filter((f) => f.endsWith('.ts')));
			const oldFiles = new Set(
				readdirSync(BINDINGS_DIR).filter((f) => f.endsWith('.ts') && f !== 'index.ts')
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

			// Remove orphaned files (types deleted from Rust)
			for (const file of oldFiles) {
				if (!newFiles.has(file)) {
					rmSync(join(BINDINGS_DIR, file));
					changed++;
				}
			}

			run(['bun', 'scripts/bindings-barrel.ts']);

			const count = newFiles.size;
			const detail = changed > 0 ? `, ${changed} changed` : ', no changes';
			process.stdout.write(c('32', '✓ bindings') + ` (${elapsed(t)}s, ${count} types${detail})\n`);
		} finally {
			rmSync(tmpDir, { recursive: true, force: true });
		}
	} else {
		process.stdout.write(c('2', '· bindings up-to-date, skipped') + '\n');
	}
}

// Regenerate JSON schemas if Rust sources are newer than exported schemas.
// These schemas are used by mod-test to detect API contract drift.
{
	const SCHEMAS_DIR = 'schemas';

	const newestSchemaMtime = existsSync(SCHEMAS_DIR)
		? newestMtime(SCHEMAS_DIR, '*.json')
		: 0;

	const stale = newestSchemaMtime === 0 || rustSrcMtime > newestSchemaMtime;
	if (stale) {
		const t = Date.now();
		process.stdout.write(c('1;36', '→ Regenerating JSON schemas (Rust sources changed)...') + '\n');
		run(['cargo', 'test', 'export_all_schemas', '--quiet'], { cwd: 'backend' });

		const count = readdirSync(SCHEMAS_DIR).filter((f) => f.endsWith('.json')).length;
		process.stdout.write(c('32', '✓ schemas') + ` (${elapsed(t)}s, ${count} types)\n`);
	} else {
		process.stdout.write(c('2', '· schemas up-to-date, skipped') + '\n');
	}
}

// Regenerate SQLx query metadata if Rust sources or migrations are newer.
{
	const SQLX_DIR = 'backend/.sqlx';

	const sqlxSrcMtime = Math.max(rustSrcMtime, newestMtime('backend/migrations', '*.sql'));

	const newestSqlxMtime = existsSync(SQLX_DIR)
		? newestMtime(SQLX_DIR, '*.json')
		: 0;

	const stale = newestSqlxMtime === 0 || sqlxSrcMtime > newestSqlxMtime;
	if (stale) {
		const t = Date.now();
		process.stdout.write(c('1;36', '→ Regenerating SQLx query metadata (sources changed)...') + '\n');
		const result = runPiped(['cargo', 'sqlx', 'prepare'], { cwd: 'backend' });
		if (result.exitCode !== 0) {
			process.stdout.write(
				c('33', '⚠ sqlx prepare failed (is the database running?)') + ` (${elapsed(t)}s)\n`
			);
			if (result.stderr) process.stderr.write(result.stderr);
		} else {
			const count = existsSync(SQLX_DIR)
				? readdirSync(SQLX_DIR).filter((f) => f.endsWith('.json')).length
				: 0;
			process.stdout.write(c('32', '✓ sqlx') + ` (${elapsed(t)}s, ${count} queries)\n`);
		}
	} else {
		process.stdout.write(c('2', '· sqlx metadata up-to-date, skipped') + '\n');
	}
}

// Regenerate GraphQL schema + gql-tada output if Rust sources are newer.
{
	const SCHEMA_PATH = 'frontend/src/lib/graphql/schema.graphql';
	const GQL_ENV_PATH = 'frontend/src/lib/graphql/graphql-env.d.ts';

	const schemaMtime = existsSync(SCHEMA_PATH) ? statSync(SCHEMA_PATH).mtimeMs : 0;
	const schemaStale = schemaMtime === 0 || rustSrcMtime > schemaMtime;

	if (schemaStale) {
		const t = Date.now();
		process.stdout.write(
			c('1;36', '→ Regenerating GraphQL schema (Rust sources changed)...') + '\n'
		);
		const result = runPiped(['cargo', 'test', '--test', 'graphql_schema', '--quiet'], {
			cwd: 'backend',
		});
		if (result.exitCode !== 0) {
			process.stdout.write(c('31', '✗ graphql schema generation failed') + '\n');
			if (result.stdout) process.stdout.write(result.stdout);
			if (result.stderr) process.stderr.write(result.stderr);
			process.exit(1);
		}
		process.stdout.write(c('32', '✓ graphql schema') + ` (${elapsed(t)}s)\n`);
	} else {
		process.stdout.write(c('2', '· graphql schema up-to-date, skipped') + '\n');
	}

	// Regenerate gql-tada introspection output if schema is newer than the env file
	const gqlEnvMtime = existsSync(GQL_ENV_PATH) ? statSync(GQL_ENV_PATH).mtimeMs : 0;
	const newSchemaMtime = existsSync(SCHEMA_PATH) ? statSync(SCHEMA_PATH).mtimeMs : 0;
	const gqlEnvStale = gqlEnvMtime === 0 || newSchemaMtime > gqlEnvMtime;

	if (gqlEnvStale) {
		const t = Date.now();
		process.stdout.write(
			c('1;36', '→ Regenerating gql-tada introspection output...') + '\n'
		);
		const result = runPiped(['bunx', 'gql-tada', 'generate', 'output'], {
			cwd: 'frontend',
		});
		if (result.exitCode !== 0) {
			process.stdout.write(c('31', '✗ gql-tada generate failed') + '\n');
			if (result.stdout) process.stdout.write(result.stdout);
			if (result.stderr) process.stderr.write(result.stderr);
			process.exit(1);
		}
		process.stdout.write(c('32', '✓ gql-tada') + ` (${elapsed(t)}s)\n`);
	} else {
		process.stdout.write(c('2', '· gql-tada output up-to-date, skipped') + '\n');
	}
}

interface Check {
	name: string;
	cmd: string[];
	cwd?: string;
	hint?: string;
	subsystem: 'frontend' | 'backend' | 'mod' | 'security';
}

const checks: Check[] = [
	// Frontend checks (3)
	{
		name: 'frontend-check',
		subsystem: 'frontend',
		cmd: ['bun', 'run', '--cwd', 'frontend', 'check']
	},
	{
		name: 'frontend-lint',
		subsystem: 'frontend',
		cmd: ['bun', 'run', '--cwd', 'frontend', 'lint']
	},
	{
		name: 'frontend-format',
		subsystem: 'frontend',
		cmd: ['bun', 'run', '--cwd', 'frontend', 'format:check'],
		hint: "Run 'bun run --cwd frontend format' to fix formatting."
	},
	// Backend checks (3)
	{
		name: 'backend-format',
		subsystem: 'backend',
		cmd: ['cargo', 'fmt', '--manifest-path', 'backend/Cargo.toml', '--', '--check'],
		hint: "Run 'cargo fmt --manifest-path backend/Cargo.toml' to fix formatting."
	},
	{
		name: 'backend-lint',
		subsystem: 'backend',
		cmd: ['cargo', 'clippy', '--manifest-path', 'backend/Cargo.toml', '--', '--deny', 'warnings']
	},
	{
		name: 'backend-test',
		subsystem: 'backend',
		cmd: [
			'cargo',
			'nextest',
			'run',
			'--manifest-path',
			'backend/Cargo.toml',
			'-E',
			'not test(export_bindings) and not test(export_graphql_sdl)'
		]
	},
	// Mod checks (4)
	{
		name: 'mod-format',
		subsystem: 'mod',
		cmd: ['./gradlew', 'spotlessCheck', 'ktlintCheck', '--quiet'],
		cwd: 'mod',
		hint: "Run 'cd mod && ./gradlew spotlessApply ktlintFormat --quiet' to fix formatting."
	},
	{
		name: 'mod-lint',
		subsystem: 'mod',
		cmd: ['./gradlew', 'detekt', '--quiet'],
		cwd: 'mod'
	},
	{
		name: 'mod-check',
		subsystem: 'mod',
		cmd: ['./gradlew', ':common:compileKotlin', ':common:compileJava', '--quiet'],
		cwd: 'mod'
	},
	{
		name: 'mod-test',
		subsystem: 'mod',
		cmd: ['./gradlew', 'test', '--quiet'],
		cwd: 'mod'
	},
	// Frontend unit tests
	{
		name: 'frontend-test',
		subsystem: 'frontend',
		cmd: ['bun', 'run', '--cwd', 'frontend', 'test:unit']
	},
	// Security audits (2)
	{
		name: 'backend-audit',
		subsystem: 'security',
		cmd: ['cargo', 'audit', '-f', 'backend/Cargo.lock']
	},
	{
		name: 'frontend-audit',
		subsystem: 'security',
		cmd: ['bun', 'scripts/audit.ts']
	}
];

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
		format: () => runPiped(['bun', 'run', '--cwd', 'frontend', 'format']),
		recheck: [
			{
				name: 'frontend-format',
				subsystem: 'frontend',
				cmd: ['bun', 'run', '--cwd', 'frontend', 'format:check']
			},
			{
				name: 'frontend-check',
				subsystem: 'frontend',
				cmd: ['bun', 'run', '--cwd', 'frontend', 'check']
			}
		]
	},
	'backend-format': {
		peers: ['backend-lint', 'backend-test'],
		format: () => runPiped(['cargo', 'fmt', '--manifest-path', 'backend/Cargo.toml']),
		recheck: [
			{
				name: 'backend-format',
				subsystem: 'backend',
				cmd: ['cargo', 'fmt', '--manifest-path', 'backend/Cargo.toml', '--', '--check']
			},
			{
				name: 'backend-lint',
				subsystem: 'backend',
				cmd: ['cargo', 'clippy', '--manifest-path', 'backend/Cargo.toml', '--', '--deny', 'warnings']
			}
		]
	},
	'mod-format': {
		peers: ['mod-lint', 'mod-check', 'mod-test'],
		format: () => runPiped(['./gradlew', 'spotlessApply', 'ktlintFormat', '--quiet'], { cwd: 'mod' }),
		recheck: [
			{
				name: 'mod-format',
				subsystem: 'mod',
				cmd: ['./gradlew', 'spotlessCheck', 'ktlintCheck', '--quiet'],
				cwd: 'mod'
			},
			{
				name: 'mod-check',
				subsystem: 'mod',
				cmd: ['./gradlew', ':common:compileKotlin', ':common:compileJava', '--quiet'],
				cwd: 'mod'
			}
		]
	}
};

const start = Date.now();
const remaining = new Set(checks.map((ch) => ch.name));

const promises = checks.map(async (check) => ({
	...check,
	...(await spawnCollect(check.cmd, start, { cwd: check.cwd }))
}));

const interval = isStderrTTY
	? setInterval(() => {
			process.stderr.write(`\r\x1b[K${elapsed(start)}s [${Array.from(remaining).join(', ')}]`);
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

for (const [fmtName, domain] of Object.entries(domains)) {
	const fmtResult = results[fmtName];
	if (!fmtResult || fmtResult.exitCode === 0) continue;
	if (!domain.peers.every((p) => results[p]?.exitCode === 0)) continue;

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
