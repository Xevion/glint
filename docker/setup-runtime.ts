#!/usr/bin/env bun
/**
 * Assembles the Minecraft + Fabric + Iris + Sodium runtime using portablemc.
 * Runs during Docker build (Stage 2) to bake everything into the image.
 */

import { existsSync, readdirSync, statSync } from "node:fs";
import { mkdir, readdir, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";
import { parseArgs } from "node:util";

const { values: args } = parseArgs({
  options: {
    "mc-version": {
      type: "string",
      default: process.env.MC_VERSION ?? "1.21.4",
    },
    "fabric-loader": {
      type: "string",
      default: process.env.FABRIC_LOADER ?? "0.16.9",
    },
    "mc-dir": { type: "string", default: process.env.MC_DIR ?? "/minecraft" },
  },
});

const MC_VERSION = args["mc-version"]!;
const FABRIC_LOADER = args["fabric-loader"]!;
const MC_DIR = args["mc-dir"]!;

function run(cmd: string[], opts?: { cwd?: string }): void {
  const result = Bun.spawnSync(cmd, {
    cwd: opts?.cwd,
    stdout: "inherit",
    stderr: "inherit",
  });
  if (!result.success) {
    throw new Error(
      `Command failed (exit ${result.exitCode}): ${cmd.join(" ")}`,
    );
  }
}

/** Convert Maven coordinate (group:artifact:version[:classifier]) to a relative JAR path. */
function mavenToPath(name: string): string | null {
  const parts = name.split(":");
  if (parts.length < 3) return null;
  const [group, artifact, version, classifier] = parts;
  const groupPath = group.replace(/\./g, "/");
  const jarName = classifier
    ? `${artifact}-${version}-${classifier}.jar`
    : `${artifact}-${version}.jar`;
  return join("libraries", groupPath, artifact, version, jarName);
}

interface VersionJson {
  id: string;
  inheritsFrom?: string;
  assetIndex?: { id: string };
  libraries: LibraryEntry[];
}

interface LibraryEntry {
  name: string;
  rules?: Array<{ action: string; os?: { name?: string } }>;
  natives?: Record<string, string>;
  downloads?: {
    classifiers?: Record<string, { path?: string }>;
  };
}

/** Check whether a library entry's rules allow it on Linux. */
function isAllowedOnLinux(rules: LibraryEntry["rules"]): boolean {
  if (!rules || rules.length === 0) return true;
  let allowed = false;
  for (const rule of rules) {
    const osName = rule.os?.name ?? "";
    if (rule.action === "allow") {
      allowed = !osName || osName === "linux";
    } else if (rule.action === "disallow" && osName === "linux") {
      allowed = false;
    }
  }
  return allowed;
}

/** Recursively walk a directory yielding file paths. */
function* walkFiles(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) yield* walkFiles(full);
    else yield full;
  }
}

console.log(
  `=== Setting up Minecraft ${MC_VERSION} + Fabric ${FABRIC_LOADER} ===`,
);

run([
  "portablemc",
  "--main-dir",
  MC_DIR,
  "start",
  `fabric:${MC_VERSION}:${FABRIC_LOADER}`,
  "--dry",
]);

console.log("=== Downloading mods from Modrinth ===");

const modsDir = join(MC_DIR, "mods");
await mkdir(modsDir, { recursive: true });

const MODRINTH_UA = "glint-docker-build/1.0 (xevion@github)";

async function downloadMod(
  slug: string,
  output: string,
  pinnedVersion?: string,
): Promise<void> {
  console.log(
    `  Fetching ${slug} for MC ${MC_VERSION} Fabric${pinnedVersion ? ` (pinned: ${pinnedVersion})` : ""}...`,
  );

  const versionsUrl = new URL(
    `https://api.modrinth.com/v2/project/${slug}/version`,
  );
  versionsUrl.searchParams.set("game_versions", JSON.stringify([MC_VERSION]));
  versionsUrl.searchParams.set("loaders", JSON.stringify(["fabric"]));

  const res = await fetch(versionsUrl, {
    headers: { "User-Agent": MODRINTH_UA },
    signal: AbortSignal.timeout(15_000),
  });
  if (!res.ok)
    throw new Error(
      `Modrinth API error for ${slug}: ${res.status} ${res.statusText}`,
    );

  const versions = (await res.json()) as Array<{
    version_number: string;
    files: Array<{ url: string }>;
  }>;

  let selected: (typeof versions)[number] | undefined;
  if (pinnedVersion) {
    selected = versions.find((v) => v.version_number === pinnedVersion);
    if (!selected) {
      const available = versions.map((v) => v.version_number).join(", ");
      throw new Error(
        `Pinned version ${pinnedVersion} not found for ${slug}. Available: ${available}`,
      );
    }
  } else {
    selected = versions[0];
  }

  if (!selected?.files.length) {
    throw new Error(
      `No Modrinth version found for ${slug} (MC ${MC_VERSION}, Fabric)`,
    );
  }

  const fileUrl = selected.files[0].url;
  console.log(`  Downloading: ${fileUrl} (${selected.version_number})`);

  const download = await fetch(fileUrl, {
    signal: AbortSignal.timeout(60_000),
  });
  if (!download.ok)
    throw new Error(`Download failed for ${slug}: ${download.status}`);

  // Buffer the full body before writing — streaming a Response directly to
  // Bun.write can stall in BuildKit containers.
  const body = await download.arrayBuffer();
  await Bun.write(output, body);
  const sizeMB = (body.byteLength / 1024 / 1024).toFixed(1);
  console.log(`  Saved: ${output} (${sizeMB} MB)`);
}

// Pin mod versions to match libs.versions.toml — mismatched versions cause block
// registry shifts that crash Iris's shader material mapping at runtime.
const mods: Array<{ slug: string; filename: string; version: string }> = [
  { slug: "fabric-api", filename: "fabric-api.jar", version: "0.116.1+1.21.4" },
  {
    slug: "architectury-api",
    filename: "architectury-api.jar",
    version: "15.0.3+fabric",
  },
  { slug: "owo-lib", filename: "owo-lib.jar", version: "0.12.19+1.21.4" },
  { slug: "iris", filename: "iris.jar", version: "1.8.8+1.21.4-fabric" },
  { slug: "sodium", filename: "sodium.jar", version: "mc1.21.4-0.6.13-fabric" },
];

// Download all mods concurrently
await Promise.all(
  mods.map(({ slug, filename, version }) =>
    downloadMod(slug, join(modsDir, filename), version),
  ),
);

console.log("=== Extracting launch metadata ===");

const versionsDir = join(MC_DIR, "versions");

// Find the Fabric version JSON
const fabricDir = readdirSync(versionsDir).find((d) => d.startsWith("fabric-"));
if (!fabricDir) throw new Error("No Fabric version directory found");

const fabricJsonPath = join(versionsDir, fabricDir, `${fabricDir}.json`);
const fabricData: VersionJson = await Bun.file(fabricJsonPath).json();
console.log(`  Fabric JSON: ${fabricJsonPath}`);

const versionId = fabricData.id;
const inherits = fabricData.inheritsFrom;

// Load vanilla JSON if inherited
let vanillaData: Partial<VersionJson> = {};
if (inherits) {
  const vanillaJsonPath = join(versionsDir, inherits, `${inherits}.json`);
  if (existsSync(vanillaJsonPath)) {
    vanillaData = await Bun.file(vanillaJsonPath).json();
    console.log(`  Vanilla JSON: ${vanillaJsonPath}`);
  } else {
    console.warn(
      `  WARNING: Inherited version JSON not found: ${vanillaJsonPath}`,
    );
  }
}

// Asset index comes from vanilla, with Fabric as fallback
const assetIndex =
  vanillaData.assetIndex?.id ?? fabricData.assetIndex?.id ?? "unknown";

// Build classpath from both version JSONs (vanilla first, then Fabric additions)
const classpathEntries: string[] = [];
const seen = new Set<string>();

for (const data of [vanillaData, fabricData]) {
  for (const lib of data.libraries ?? []) {
    if (seen.has(lib.name)) continue;
    seen.add(lib.name);

    if (!isAllowedOnLinux(lib.rules)) continue;

    const relPath = mavenToPath(lib.name);
    if (!relPath) continue;

    const jarPath = join(MC_DIR, relPath);
    if (existsSync(jarPath)) {
      classpathEntries.push(jarPath);
    } else {
      console.warn(`  WARNING: Missing library: ${jarPath}`);
    }
  }
}

// Locate the client JAR — Fabric's MinecraftGameProvider needs it on the classpath
let clientJar: string | null = null;
if (inherits) {
  const candidates = [
    join(versionsDir, inherits, `${inherits}.jar`),
    join(versionsDir, versionId, `${versionId}.jar`),
  ];
  clientJar = candidates.find((c) => existsSync(c)) ?? null;

  // Last resort: find any JAR > 10MB in versions/ (client JAR is ~70MB)
  if (!clientJar) {
    for (const fpath of walkFiles(versionsDir)) {
      if (fpath.endsWith(".jar") && statSync(fpath).size > 10_000_000) {
        clientJar = fpath;
        console.warn(`  Found client JAR by size heuristic: ${fpath}`);
        break;
      }
    }
  }
}

if (clientJar) {
  classpathEntries.push(clientJar);
  const sizeMB = (statSync(clientJar).size / 1024 / 1024).toFixed(1);
  console.log(`  Client JAR: ${clientJar} (${sizeMB} MB)`);
} else {
  console.error("  ERROR: Minecraft client JAR not found!");
  console.error(`  Searched in: ${versionsDir}`);
  for (const fpath of walkFiles(versionsDir)) {
    if (fpath.endsWith(".jar")) {
      console.error(`    JAR: ${fpath} (${statSync(fpath).size} bytes)`);
    }
  }
  process.exit(1);
}

// Find or create natives directory
const nativesCandidates = [
  join(MC_DIR, "natives"),
  join(versionsDir, versionId, "natives"),
  ...(inherits ? [join(versionsDir, inherits, "natives")] : []),
];
let nativesDir =
  nativesCandidates.find((d) => existsSync(d) && statSync(d).isDirectory()) ??
  null;

if (!nativesDir) {
  // Extract natives from library JARs that have 'natives-linux' classifiers
  nativesDir = join(MC_DIR, "natives");
  await mkdir(nativesDir, { recursive: true });

  for (const data of [vanillaData, fabricData]) {
    for (const lib of data.libraries ?? []) {
      const nativeKey = lib.natives?.linux;
      const classifiers = lib.downloads?.classifiers;
      if (!nativeKey || !classifiers?.[nativeKey]) continue;

      const nativePath = join(
        MC_DIR,
        "libraries",
        classifiers[nativeKey].path ?? "",
      );
      if (!existsSync(nativePath)) continue;

      console.log(`  Extracting natives from: ${basename(nativePath)}`);
      const zip = new Bun.ZipReader(await Bun.file(nativePath).arrayBuffer());
      for (const entry of zip.entries) {
        if (entry.name.endsWith(".so")) {
          const outPath = join(nativesDir, entry.name);
          const dir = join(
            nativesDir,
            entry.name.split("/").slice(0, -1).join("/"),
          );
          if (dir !== nativesDir) await mkdir(dir, { recursive: true });
          const data = entry.decompress();
          await Bun.write(outPath, data);
        }
      }
      zip.close();
    }
  }
}

// Write metadata files consumed by the entrypoint
const metadataFiles: Record<string, string> = {
  "version-id.txt": versionId,
  "asset-index.txt": assetIndex,
  "classpath.txt": classpathEntries.join(":"),
  "natives-dir.txt": nativesDir,
};

await Promise.all(
  Object.entries(metadataFiles).map(([name, content]) =>
    writeFile(join(MC_DIR, name), content),
  ),
);

console.log();
console.log(`  Version ID:        ${versionId}`);
console.log(`  Asset index:       ${assetIndex}`);
console.log(`  Classpath entries: ${classpathEntries.length}`);
console.log(`  Natives dir:       ${nativesDir}`);

console.log("=== Runtime setup complete ===");
console.log("  Mods:");
for (const f of await readdir(modsDir)) {
  const stat = statSync(join(modsDir, f));
  const sizeMB = (stat.size / 1024 / 1024).toFixed(1);
  console.log(`    ${f} (${sizeMB} MB)`);
}
