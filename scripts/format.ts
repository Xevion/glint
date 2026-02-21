/**
 * Auto-format code across subsystems.
 *
 * Usage: bun scripts/format.ts [targets...]
 *
 * Targets scope formatting to specific subsystems (comma-delimited):
 *   backend, frontend, mod (and aliases: b, f, m, rust, web, etc.)
 *   Omit targets to format everything.
 */

import { getCommand } from './lib/commands';
import { c } from './lib/fmt';
import { run, runPiped } from './lib/proc';
import { isAll, resolveTargets, targetLabel } from './lib/targets';

const argv = process.argv.slice(2);

if (argv[0] === '--help' || argv[0] === '-h') {
	console.log(`Usage: bun scripts/format.ts [targets...]

Auto-formats code for one or more subsystems.

Targets (comma-delimited, omit for all):
  backend (b, back, rust)     cargo fmt
  frontend (f, front, web)    Biome format
  mod (m, minecraft)          Spotless + ktlint

Examples:
  bun scripts/format.ts              Format everything
  bun scripts/format.ts backend      cargo fmt only
  bun scripts/format.ts f,m          Frontend + mod`);
	process.exit(0);
}

const targets = resolveTargets(argv);

if (!isAll(targets)) {
	process.stdout.write(c('1;36', `→ Formatting: ${targetLabel(targets)}`) + '\n');
}

if (targets.subsystems.has('frontend')) {
	run(getCommand('frontend', 'format-apply').cmd);
}
if (targets.subsystems.has('backend')) {
	run(getCommand('backend', 'format-apply').cmd);
}
if (targets.subsystems.has('mod')) {
	const def = getCommand('mod', 'format-apply');
	runPiped(def.cmd, { cwd: def.cwd });
}
