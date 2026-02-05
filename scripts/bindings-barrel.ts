/**
 * Generate the barrel (index.ts) file for TypeScript bindings and format them.
 *
 * Usage: bun scripts/bindings-barrel.ts
 */

import { readdirSync, writeFileSync } from "fs";
import { runPiped } from "./lib/proc";

const BINDINGS_DIR = "frontend/src/lib/bindings";

const types = readdirSync(BINDINGS_DIR)
  .filter((f) => f.endsWith(".ts") && f !== "index.ts")
  .map((f) => f.replace(/\.ts$/, ""))
  .sort();

writeFileSync(
  `${BINDINGS_DIR}/index.ts`,
  "// Auto-generated barrel file — do not edit manually.\n" +
    "// Regenerate with: cd backend && cargo test export_bindings\n" +
    types.map((t) => `export type { ${t} } from "./${t}";`).join("\n") +
    "\n",
);

runPiped(["bun", "run", "--cwd", "frontend", "format", "src/lib/bindings/"]);

console.log(`✓ Generated barrel file (${types.length} types)`);
