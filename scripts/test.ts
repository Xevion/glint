/**
 * Run project tests across all subsystems.
 *
 * Usage: bun scripts/test.ts [web|rust|mod|agent|<nextest filter args>]
 */

import { run, ProcessGroup } from "./lib/proc";

const input = process.argv.slice(2).join(" ").trim();

if (input === "web") {
  // Frontend tests only
  run(["bun", "run", "--cwd", "frontend", "test"]);
} else if (input === "rust") {
  // Backend tests only
  run(["cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml"]);
} else if (input === "mod") {
  // Mod tests only
  run(["sh", "-c", "cd mod && ./gradlew test --quiet"]);
} else if (input === "agent") {
  // Agent tests only
  run(["cargo", "nextest", "run", "--manifest-path", "agent/Cargo.toml"]);
} else if (input === "") {
  // All tests in parallel
  const group = new ProcessGroup();
  group.spawn(["bun", "run", "--cwd", "frontend", "test"]);
  group.spawn(["cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml"]);
  group.spawn(["sh", "-c", "cd mod && ./gradlew test --quiet"]);
  group.spawn(["cargo", "nextest", "run", "--manifest-path", "agent/Cargo.toml"]);
  const code = await group.waitForFirst();
  process.exit(code);
} else {
  // Assume nextest filter args for backend tests
  run(["cargo", "nextest", "run", "--manifest-path", "backend/Cargo.toml", ...input.split(/\s+/)]);
}
