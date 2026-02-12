/**
 * Run project tests across all subsystems.
 *
 * Usage: bun scripts/test.ts [web|web-e2e|rust|mod|<nextest filter args>]
 */

import { run, ProcessGroup } from "./lib/proc";

const argv = process.argv.slice(2);
const subcommand = argv[0] ?? "";

if (subcommand === "--help" || subcommand === "-h") {
	console.log(`Usage: bun scripts/test.ts [target]

Runs tests for one or all subsystems.

Targets:
  web       Frontend unit tests (Vitest)
  web-e2e   Frontend E2E tests (Playwright)
  rust      Backend tests (cargo nextest)
  mod       Mod tests (Gradle)
  <filter>  Nextest filter args (passed to backend tests)

No target runs all unit tests in parallel.`);
	process.exit(0);
}

if (subcommand === "web") {
	run(["bun", "run", "--cwd", "frontend", "test:unit"]);
} else if (subcommand === "web-e2e") {
	run(["bun", "run", "--cwd", "frontend", "test:e2e"]);
} else if (subcommand === "rust") {
	run(["cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml"]);
} else if (subcommand === "mod") {
	run(["./gradlew", "test", "--quiet"], { cwd: "mod" });
} else if (subcommand === "") {
	const group = new ProcessGroup();
	group.spawn(["bun", "run", "--cwd", "frontend", "test:unit"]);
	group.spawn(["cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml"]);
	group.spawn(["./gradlew", "test", "--quiet"], { cwd: "mod" });
	const code = await group.waitForAll();
	process.exit(code);
} else {
	// Pass remaining args as nextest filter
	run(["cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml", ...argv]);
}
