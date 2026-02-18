/**
 * Run `bun audit` with a shared set of ignored advisories.
 *
 * Both `scripts/check.ts` and `.github/workflows/ci.yml` call this
 * script so the ignore list stays in one place.
 */

const IGNORED_ADVISORIES = [
	"GHSA-2g4f-4pwh-qvx6", // ajv ReDoS — transitive via eslint, no fix available
];

const cmd = [
	"bun", "audit", "--audit-level=moderate",
	...IGNORED_ADVISORIES.map((id) => `--ignore=${id}`),
];

const proc = Bun.spawnSync(cmd, {
	stdio: ["ignore", "inherit", "inherit"],
	cwd: "frontend",
});

process.exit(proc.exitCode);
