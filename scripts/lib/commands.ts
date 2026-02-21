/**
 * Centralized command registry for Glint project scripts.
 *
 * Maps (subsystem, action) tuples to command definitions, eliminating
 * duplication across check.ts, format.ts, lint.ts, pre-commit.ts, and test.ts.
 */

import type { Subsystem } from './targets';

export type Action = 'format-check' | 'format-apply' | 'lint' | 'type-check' | 'test' | 'compile';

export interface CommandDef {
	/** Command array [program, ...args] */
	cmd: string[];
	/** Working directory relative to project root */
	cwd?: string;
	/** Human-readable description */
	description: string;
}

const REGISTRY: Record<Subsystem, Partial<Record<Action, CommandDef>>> = {
	frontend: {
		'format-check': {
			cmd: ['bun', 'run', '--cwd', 'frontend', 'format:check'],
			description: 'Biome format check',
		},
		'format-apply': {
			cmd: ['bun', 'run', '--cwd', 'frontend', 'format'],
			description: 'Biome format',
		},
		lint: {
			cmd: ['bun', 'run', '--cwd', 'frontend', 'lint'],
			description: 'Biome lint',
		},
		'type-check': {
			cmd: ['bun', 'run', '--cwd', 'frontend', 'check'],
			description: 'SvelteKit type check',
		},
		test: {
			cmd: ['bun', 'run', '--cwd', 'frontend', 'test:unit'],
			description: 'Vitest unit tests',
		},
	},
	backend: {
		'format-check': {
			cmd: ['cargo', 'fmt', '--manifest-path', 'backend/Cargo.toml', '--', '--check'],
			description: 'cargo fmt check',
		},
		'format-apply': {
			cmd: ['cargo', 'fmt', '--manifest-path', 'backend/Cargo.toml'],
			description: 'cargo fmt',
		},
		lint: {
			cmd: ['cargo', 'clippy', '--manifest-path', 'backend/Cargo.toml', '--', '--deny', 'warnings'],
			description: 'Clippy',
		},
		test: {
			cmd: ['cargo', 'nextest', 'run', '--manifest-path', 'backend/Cargo.toml'],
			description: 'cargo nextest',
		},
	},
	mod: {
		'format-check': {
			cmd: ['./gradlew', 'spotlessCheck', 'ktlintCheck', '--quiet'],
			cwd: 'mod',
			description: 'Spotless + ktlint check',
		},
		'format-apply': {
			cmd: ['./gradlew', 'spotlessApply', 'ktlintFormat', '--quiet'],
			cwd: 'mod',
			description: 'Spotless + ktlint format',
		},
		lint: {
			cmd: ['./gradlew', 'detekt', '--quiet'],
			cwd: 'mod',
			description: 'Detekt',
		},
		compile: {
			cmd: ['./gradlew', ':common:compileKotlin', ':common:compileJava', '--quiet'],
			cwd: 'mod',
			description: 'Kotlin/Java compile check',
		},
		test: {
			cmd: ['./gradlew', 'test', '--quiet'],
			cwd: 'mod',
			description: 'Gradle tests',
		},
	},
};

/**
 * Look up a command definition by subsystem and action.
 *
 * @throws Error if the (subsystem, action) combination doesn't exist
 */
export function getCommand(subsystem: Subsystem, action: Action): CommandDef {
	const def = REGISTRY[subsystem]?.[action];
	if (!def) throw new Error(`No command registered for (${subsystem}, ${action})`);
	return def;
}
