/**
 * Setup fixtures for offline agent testing.
 *
 * Downloads shader packs from Modrinth and sets up the local .fixtures directory
 * for use with `just dev-agent --offline`.
 *
 * Usage: bun scripts/setup-fixtures.ts [options] [shaders...]
 *
 * Options:
 *   --all           Download all configured shaders
 *   --list          List available shader configs
 *   --force         Re-download even if files exist
 *   --help          Show this help
 *
 * Examples:
 *   bun scripts/setup-fixtures.ts bsl complementary
 *   bun scripts/setup-fixtures.ts --all
 *   bun scripts/setup-fixtures.ts --list
 */

import { existsSync, mkdirSync, readdirSync, copyFileSync } from "fs";
import { join, dirname } from "path";
import { c } from "./lib/fmt";
import { fileURLToPath } from "url";

// Resolve paths relative to project root
const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = join(__dirname, "..");

// Paths
const AGENT_DIR = join(PROJECT_ROOT, "agent");
const FIXTURES_SRC = join(AGENT_DIR, "fixtures-src");
const FIXTURES_DIR = join(AGENT_DIR, ".fixtures");
const SHADERS_CONFIG = join(FIXTURES_SRC, "shaders.json");
const SCENES_SRC_DIR = join(FIXTURES_SRC, "scenes");

// Modrinth API
const MODRINTH_API = "https://api.modrinth.com/v2";
const USER_AGENT = "glint-fixture-setup/1.0 (https://github.com/xevion/glint)";

interface ShaderConfig {
	name: string;
	modrinth_id: string;
	description: string;
}

interface ShadersConfig {
	shaders: Record<string, ShaderConfig>;
}

interface ModrinthVersion {
	id: string;
	version_number: string;
	name: string;
	files: Array<{
		url: string;
		filename: string;
		hashes: { sha512: string };
		primary: boolean;
	}>;
	game_versions: string[];
	loaders: string[];
}

interface ManifestShader {
	id: string;
	name: string;
	version_id: string;
	version: string;
	file: string;
	file_hash: string;
}

interface ManifestScene {
	id: string;
	name: string;
	world_slug: string;
	definition_file: string;
}

interface ManifestWorld {
	id: string;
	name: string;
	file: string;
	file_hash?: string;
}

interface Manifest {
	shaders: Record<string, ManifestShader>;
	worlds: Record<string, ManifestWorld>;
	scenes: Record<string, ManifestScene>;
}

// Parse args
const args = process.argv.slice(2);
const showHelp = args.includes("--help") || args.includes("-h");
const showList = args.includes("--list");
const downloadAll = args.includes("--all");
const force = args.includes("--force");
const shaderSlugs = args.filter((a) => !a.startsWith("-"));

if (showHelp) {
	console.log(`
${c("1;36", "Setup Fixtures")} - Download shaders for offline agent testing

${c("33", "Usage:")}
  bun scripts/setup-fixtures.ts [options] [shaders...]

${c("33", "Options:")}
  --all           Download all configured shaders
  --list          List available shader configs
  --force         Re-download even if files exist
  --help          Show this help

${c("33", "Examples:")}
  bun scripts/setup-fixtures.ts bsl complementary
  bun scripts/setup-fixtures.ts --all
  bun scripts/setup-fixtures.ts --list
`);
	process.exit(0);
}

// Load shader configs
async function loadShaderConfigs(): Promise<ShadersConfig> {
	const file = Bun.file(SHADERS_CONFIG);
	if (!(await file.exists())) {
		console.error(c("31", `✗ Shader config not found: ${SHADERS_CONFIG}`));
		process.exit(1);
	}
	return JSON.parse(await file.text());
}

// Modrinth API helpers
async function modrinthGet<T>(path: string): Promise<T> {
	const response = await fetch(`${MODRINTH_API}${path}`, {
		headers: { "User-Agent": USER_AGENT },
	});
	if (!response.ok) {
		throw new Error(`Modrinth API error: ${response.status} ${response.statusText}`);
	}
	return response.json();
}

async function getLatestShaderVersion(modrinthId: string): Promise<ModrinthVersion | null> {
	const versions = await modrinthGet<ModrinthVersion[]>(
		`/project/${modrinthId}/version?loaders=["iris"]`,
	);

	// Find the latest version that supports iris
	const irisVersions = versions.filter((v) => v.loaders.includes("iris"));

	if (irisVersions.length === 0) {
		// Fall back to any version
		return versions[0] ?? null;
	}

	return irisVersions[0];
}

async function downloadFile(url: string, destPath: string): Promise<void> {
	const response = await fetch(url, {
		headers: { "User-Agent": USER_AGENT },
	});
	if (!response.ok) {
		throw new Error(`Download failed: ${response.status} ${response.statusText}`);
	}
	const buffer = await response.arrayBuffer();
	await Bun.write(destPath, buffer);
}

// Compute SHA-256 hash of a file
async function computeFileHash(filePath: string): Promise<string> {
	const file = Bun.file(filePath);
	const buffer = await file.arrayBuffer();
	const hash = new Bun.CryptoHasher("sha256").update(buffer).digest("hex");
	return `sha256:${hash}`;
}

// Setup directory structure
function ensureDirectories(): void {
	for (const dir of [
		FIXTURES_DIR,
		join(FIXTURES_DIR, "shaders"),
		join(FIXTURES_DIR, "worlds"),
		join(FIXTURES_DIR, "scenes"),
	]) {
		if (!existsSync(dir)) {
			mkdirSync(dir, { recursive: true });
		}
	}
}

// Download a shader from Modrinth
async function downloadShader(
	slug: string,
	config: ShaderConfig,
	manifest: Manifest,
): Promise<boolean> {
	console.log(c("36", `\n→ ${config.name} (${slug})`));

	// Check if already downloaded (unless force)
	const existing = manifest.shaders[slug];
	if (existing && !force) {
		const filePath = join(FIXTURES_DIR, existing.file);
		if (existsSync(filePath)) {
			console.log(c("33", `  Skipping: already downloaded (use --force to re-download)`));
			return true;
		}
	}

	try {
		// Get latest version from Modrinth
		console.log(c("2;37", `  Fetching version info from Modrinth...`));
		const version = await getLatestShaderVersion(config.modrinth_id);
		if (!version) {
			console.error(c("31", `  ✗ No versions found for ${config.modrinth_id}`));
			return false;
		}

		// Find the primary file (shader zip)
		const file = version.files.find((f) => f.primary) ?? version.files[0];
		if (!file) {
			console.error(c("31", `  ✗ No files in version ${version.version_number}`));
			return false;
		}

		const destFile = `shaders/${slug}-${version.version_number}.zip`;
		const destPath = join(FIXTURES_DIR, destFile);

		// Download
		console.log(c("2;37", `  Downloading ${file.filename}...`));
		await downloadFile(file.url, destPath);

		// Compute hash
		const hash = await computeFileHash(destPath);

		// Update manifest
		manifest.shaders[slug] = {
			id: `fixture-${slug}`,
			name: config.name,
			version_id: `fixture-${slug}-${version.version_number}`,
			version: version.version_number,
			file: destFile,
			file_hash: hash,
		};

		console.log(c("32", `  ✓ Downloaded: ${version.version_number}`));
		return true;
	} catch (err) {
		console.error(c("31", `  ✗ Failed: ${err}`));
		return false;
	}
}

// Copy scene definitions from fixtures-src to .fixtures
async function copySceneDefinitions(manifest: Manifest): Promise<void> {
	console.log(c("36", "\n→ Copying scene definitions..."));

	if (!existsSync(SCENES_SRC_DIR)) {
		console.log(c("33", "  No scene definitions found in fixtures-src/scenes/"));
		return;
	}

	const sceneFiles = readdirSync(SCENES_SRC_DIR).filter((f) => f.endsWith(".json"));

	for (const file of sceneFiles) {
		const srcPath = join(SCENES_SRC_DIR, file);
		const destPath = join(FIXTURES_DIR, "scenes", file);

		copyFileSync(srcPath, destPath);

		// Parse to extract scene info for manifest
		const content = JSON.parse(await Bun.file(srcPath).text());
		const worldSlug = content.world ?? file.replace(".json", "");

		// Add each scene from the collection to manifest
		for (const scene of content.scenes ?? []) {
			manifest.scenes[scene.id] = {
				id: `fixture-${scene.id}`,
				name: scene.name,
				world_slug: worldSlug,
				definition_file: `scenes/${file}`,
			};
		}

		// Track the world reference
		if (worldSlug && !manifest.worlds[worldSlug]) {
			manifest.worlds[worldSlug] = {
				id: `fixture-${worldSlug}`,
				name: worldSlug,
				file: `worlds/${worldSlug}.zip`,
			};
		}
	}

	console.log(c("32", `  ✓ Copied ${sceneFiles.length} scene collection(s)`));
}

// Check/setup test world
async function setupTestWorld(manifest: Manifest): Promise<void> {
	console.log(c("36", "\n→ Checking test world..."));

	// Look for any world in the manifest
	const worldSlugs = Object.keys(manifest.worlds);
	if (worldSlugs.length === 0) {
		console.log(c("33", "  No worlds referenced by scenes"));
		return;
	}

	for (const slug of worldSlugs) {
		const world = manifest.worlds[slug];
		const worldPath = join(FIXTURES_DIR, world.file);

		if (existsSync(worldPath)) {
			// Compute hash if not present
			if (!world.file_hash) {
				world.file_hash = await computeFileHash(worldPath);
			}
			console.log(c("32", `  ✓ World exists: ${slug}`));
		} else {
			// Prompt user to provide world
			console.log(
				c(
					"33",
					`
┌─────────────────────────────────────────────────────────────────────┐
│  Missing test world: ${slug.padEnd(46)} │
│                                                                     │
│  Please provide a Minecraft world for testing:                      │
│                                                                     │
│  1. Launch Minecraft 1.21.4 and create/open a world                 │
│  2. Build interesting scenes at the coordinates in:                 │
│     ${join(FIXTURES_DIR, "scenes").padEnd(56)} │
│  3. Save and exit Minecraft                                         │
│  4. Zip your saves/<world> folder and place it at:                  │
│     ${worldPath.padEnd(56)} │
│                                                                     │
│  Then re-run this script.                                           │
└─────────────────────────────────────────────────────────────────────┘
`,
				),
			);
		}
	}
}

// Write manifest
async function writeManifest(manifest: Manifest): Promise<void> {
	const manifestPath = join(FIXTURES_DIR, "manifest.json");
	await Bun.write(manifestPath, JSON.stringify(manifest, null, 2));
	console.log(c("32", `\n✓ Wrote manifest: ${manifestPath}`));
}

// Main
async function main(): Promise<void> {
	console.log(c("1;36", "=== Glint Fixture Setup ===\n"));

	const config = await loadShaderConfigs();
	const availableSlugs = Object.keys(config.shaders);

	// List mode
	if (showList) {
		console.log(c("33", "Available shaders:\n"));
		for (const [slug, shader] of Object.entries(config.shaders)) {
			console.log(`  ${c("36", slug.padEnd(20))} ${shader.name}`);
			console.log(`  ${" ".repeat(20)} ${c("2;37", shader.description)}\n`);
		}
		process.exit(0);
	}

	// Determine which shaders to download
	let toDownload: string[];
	if (downloadAll) {
		toDownload = availableSlugs;
	} else if (shaderSlugs.length > 0) {
		// Validate slugs
		const invalid = shaderSlugs.filter((s) => !availableSlugs.includes(s));
		if (invalid.length > 0) {
			console.error(c("31", `Unknown shader(s): ${invalid.join(", ")}`));
			console.error(c("33", `Available: ${availableSlugs.join(", ")}`));
			process.exit(1);
		}
		toDownload = shaderSlugs;
	} else {
		// No shaders specified - show usage
		console.log(c("33", "No shaders specified. Usage:\n"));
		console.log(`  bun scripts/setup-fixtures.ts <shader> [shader...]`);
		console.log(`  bun scripts/setup-fixtures.ts --all`);
		console.log(`  bun scripts/setup-fixtures.ts --list\n`);
		console.log(c("33", `Available shaders: ${availableSlugs.join(", ")}`));
		process.exit(0);
	}

	// Setup directories
	ensureDirectories();

	// Load existing manifest or create new
	const manifestPath = join(FIXTURES_DIR, "manifest.json");
	let manifest: Manifest;
	if (existsSync(manifestPath)) {
		manifest = JSON.parse(await Bun.file(manifestPath).text());
	} else {
		manifest = { shaders: {}, worlds: {}, scenes: {} };
	}

	// Download shaders
	console.log(c("36", `Downloading ${toDownload.length} shader(s)...`));
	let success = 0;
	let failed = 0;

	for (const slug of toDownload) {
		const shaderConfig = config.shaders[slug];
		const ok = await downloadShader(slug, shaderConfig, manifest);
		if (ok) success++;
		else failed++;
	}

	// Copy scene definitions
	await copySceneDefinitions(manifest);

	// Setup world
	await setupTestWorld(manifest);

	// Write manifest
	await writeManifest(manifest);

	// Summary
	console.log(c("1;36", "\n=== Summary ==="));
	console.log(`  Shaders: ${c("32", String(success))} downloaded, ${c("31", String(failed))} failed`);
	console.log(`  Scenes:  ${c("32", String(Object.keys(manifest.scenes).length))} defined`);
	console.log(`  Worlds:  ${c("32", String(Object.keys(manifest.worlds).length))} referenced`);

	if (failed > 0) {
		process.exit(1);
	}
}

main().catch((err) => {
	console.error(c("31", `Error: ${err.message}`));
	process.exit(1);
});
