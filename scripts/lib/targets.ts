/**
 * Shared target resolution for Glint's unified command pattern.
 *
 * All core commands (check, test, lint, format) accept optional targets
 * to scope work to specific subsystems. Targets are comma-delimited
 * and resolved through an alias table.
 *
 * Usage: `just check backend,frontend` or `just test b,f`
 */

export type Subsystem = 'frontend' | 'backend' | 'mod';

const ALIASES: Record<string, Subsystem | 'e2e'> = {
	frontend: 'frontend',
	front: 'frontend',
	web: 'frontend',
	f: 'frontend',

	backend: 'backend',
	back: 'backend',
	rust: 'backend',
	b: 'backend',

	mod: 'mod',
	minecraft: 'mod',
	m: 'mod',

	e2e: 'e2e',
};

const ALL_SUBSYSTEMS: Subsystem[] = ['frontend', 'backend', 'mod'];

export interface ResolvedTargets {
	subsystems: Set<Subsystem>;
	e2e: boolean;
}

/**
 * Resolve CLI arguments into a set of subsystems and flags.
 *
 * Accepts comma-delimited target names that are resolved through an
 * alias table. Empty input selects all subsystems.
 *
 * @param argv - Arguments from process.argv (after flags are stripped)
 * @returns Resolved subsystems and whether E2E was requested
 *
 * @example
 * resolveTargets([])              // { subsystems: all, e2e: false }
 * resolveTargets(["backend"])     // { subsystems: {backend}, e2e: false }
 * resolveTargets(["web,mod"])     // { subsystems: {frontend, mod}, e2e: false }
 * resolveTargets(["b", "m"])      // { subsystems: {backend, mod}, e2e: false }
 * resolveTargets(["e2e"])         // { subsystems: {frontend}, e2e: true }
 */
export function resolveTargets(argv: string[]): ResolvedTargets {
	if (argv.length === 0) {
		return { subsystems: new Set(ALL_SUBSYSTEMS), e2e: false };
	}

	const tokens = argv.flatMap((arg) => arg.split(',').map((s) => s.trim().toLowerCase())).filter(Boolean);

	const subsystems = new Set<Subsystem>();
	let e2e = false;

	for (const token of tokens) {
		const resolved = ALIASES[token];
		if (resolved === undefined) {
			const valid = [...new Set(Object.values(ALIASES))].sort().join(', ');
			console.error(`Unknown target: '${token}'\nValid targets: ${valid}`);
			console.error(`\nAliases: ${Object.entries(ALIASES).map(([k, v]) => `${k} → ${v}`).join(', ')}`);
			process.exit(1);
		}
		if (resolved === 'e2e') {
			e2e = true;
			subsystems.add('frontend');
		} else {
			subsystems.add(resolved);
		}
	}

	return { subsystems, e2e };
}

/** Check if all subsystems are selected (no filtering needed). */
export function isAll(targets: ResolvedTargets): boolean {
	return targets.subsystems.size === ALL_SUBSYSTEMS.length;
}

/** Human-readable label for the selected targets. */
export function targetLabel(targets: ResolvedTargets): string {
	if (isAll(targets) && !targets.e2e) return 'all';
	const parts = [...targets.subsystems].sort();
	if (targets.e2e) parts.push('e2e');
	return parts.join(', ');
}
