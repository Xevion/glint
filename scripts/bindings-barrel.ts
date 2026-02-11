/**
 * Generate the barrel (index.ts) file for TypeScript bindings.
 *
 * Usage: bun scripts/bindings-barrel.ts
 */

import { existsSync, readFileSync, readdirSync, writeFileSync } from "fs";

const BINDINGS_DIR = "frontend/src/lib/bindings";

const types = readdirSync(BINDINGS_DIR)
  .filter((f) => f.endsWith(".ts") && f !== "index.ts")
  .map((f) => f.replace(/\.ts$/, ""))
  .sort();

const barrelContent =
  "// Auto-generated barrel file — do not edit manually.\n" +
  "// Regenerate with: cd backend && cargo test export_bindings\n" +
  types.map((t) => `export type { ${t} } from "./${t}";`).join("\n") +
  "\n";

const barrelPath = `${BINDINGS_DIR}/index.ts`;
const existing = existsSync(barrelPath) ? readFileSync(barrelPath, "utf-8") : null;

if (existing !== barrelContent) {
  writeFileSync(barrelPath, barrelContent);
  console.log(`✓ Generated barrel file (${types.length} types)`);
} else {
  console.log(`✓ Barrel file unchanged (${types.length} types)`);
}
