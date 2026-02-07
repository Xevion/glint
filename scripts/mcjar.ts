#!/usr/bin/env bun

import { spawn } from "bun";
import * as fs from "fs/promises";
import os from "os";
import path from "path";

type JarKind = "sources" | "merged";

// Loom stores decompiled sources in the global Gradle cache, while
// per-project remapped/injected merged JARs live under mod/.gradle/.
// Search global first (has sources), then fall back to project-local.
const SEARCH_ROOTS = [
  path.join(
    os.homedir(),
    ".gradle/caches/fabric-loom/minecraftMaven/net/minecraft"
  ),
  "mod/.gradle/loom-cache/minecraftMaven/net/minecraft",
];

type Config = {
  version?: string;
  context: number;
  sourcesJar?: string;
  mergedJar?: string;
};

const config: Config = {
  version: process.env.MC_VERSION,
  context: 3,
  sourcesJar: process.env.MC_SOURCES_JAR,
  mergedJar: process.env.MC_MERGED_JAR,
};

const [command, parsedArgs] = parseArgs(Bun.argv.slice(2));
const jarCache = new Map<JarKind, string>();

function parseArgs(argv: string[]): [string | undefined, string[]] {
  const positional: string[] = [];

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    switch (arg) {
      case "--version":
        config.version = argv[++i];
        break;
      case "--context":
        config.context = Number(argv[++i] ?? config.context);
        break;
      case "--sources":
        config.sourcesJar = argv[++i];
        break;
      case "--merged":
        config.mergedJar = argv[++i];
        break;
      default:
        positional.push(arg);
        break;
    }
  }

  return [positional.shift(), positional];
}

function usage(): void {
  console.log(`Usage: bun scripts/mcjar.ts <command> [args]

Commands:
  list <path>                 List entries in sources jar matching path/glob
  cat <file>                  Print file from sources jar
  grep <pattern> <file>       Search pattern in file with context (--context N)
  grep-all <pattern> <glob>   Search across files in glob
  asset <file>                Print asset from merged jar
  asset-list <path>           List assets matching path/glob

Options:
  --version <ver>             Filter jars by version substring
  --sources <path>            Override sources jar path
  --merged <path>             Override merged jar path
  --context <n>               Lines of context for grep (default 3)

Environment overrides:
  MC_VERSION, MC_SOURCES_JAR, MC_MERGED_JAR
`);
}

async function runCommand(
  cmd: string,
  args: string[]
): Promise<{
  code: number;
  stdout: string;
  stderr: string;
}> {
  const proc = spawn([cmd, ...args], { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, code] = await Promise.all([
    proc.stdout ? new Response(proc.stdout).text() : "",
    proc.stderr ? new Response(proc.stderr).text() : "",
    proc.exited,
  ]);
  return { code, stdout, stderr };
}

async function findJar(kind: JarKind): Promise<string> {
  if (jarCache.has(kind)) {
    return jarCache.get(kind)!;
  }

  const override = kind === "sources" ? config.sourcesJar : config.mergedJar;
  if (override) {
    jarCache.set(kind, override);
    return override;
  }

  const pattern =
    kind === "sources" ? "*-sources.jar" : "minecraft-merged-*.jar";
  const glob = new Bun.Glob(`**/${pattern}`);
  const candidates: string[] = [];

  for (const root of SEARCH_ROOTS) {
    try {
      await fs.access(root);
    } catch {
      continue;
    }
    for await (const match of glob.scan(root)) {
      const resolved = path.isAbsolute(match) ? match : path.join(root, match);
      candidates.push(resolved);
    }
    // Prefer the first root that has results (global cache has more
    // complete JARs than project-local Architectury-injected variants)
    if (candidates.length > 0) break;
  }

  const filtered = config.version
    ? candidates.filter((p) => p.includes(config.version!))
    : candidates;

  if (filtered.length === 0) {
    throw new Error(
      `No ${kind} jar found${
        config.version ? ` for version ${config.version}` : ""
      }.`
    );
  }

  const sorted = await sortByMtime(filtered);
  const jar = sorted[0];
  jarCache.set(kind, jar);
  return jar;
}

async function sortByMtime(files: string[]): Promise<string[]> {
  const stats = await Promise.all(
    files.map(async (f) => ({
      file: f,
      mtime: (await fs.stat(f)).mtimeMs,
    }))
  );
  return stats.sort((a, b) => b.mtime - a.mtime).map((s) => s.file);
}

async function readEntry(jar: string, entry: string): Promise<string> {
  const { code, stdout, stderr } = await runCommand("unzip", [
    "-p",
    jar,
    entry,
  ]);
  if (code !== 0) {
    throw new Error(stderr || `Failed to read ${entry} from ${jar}`);
  }
  return stdout;
}

async function listEntries(jar: string, filter: string): Promise<string[]> {
  const { code, stdout, stderr } = await runCommand("unzip", [
    "-Z",
    "-1",
    jar,
    filter,
  ]);
  if (code !== 0) {
    throw new Error(stderr || `No entries matching ${filter}`);
  }
  return stdout.split("\n").filter(Boolean);
}

async function findSimilarPaths(jar: string, target: string): Promise<string[]> {
  // Get all entries from JAR
  const { code, stdout } = await runCommand("unzip", ["-Z", "-1", jar]);
  if (code !== 0) return [];

  const allEntries = stdout.split("\n").filter(Boolean);
  
  // Normalize the target path
  const normalized = target.replace(/\/+$/, ""); // remove trailing slashes
  
  // Find entries that start with the target path
  const matches = allEntries.filter((entry) => 
    entry.startsWith(normalized)
  );

  // If no matches, try to find parent directory suggestions
  if (matches.length === 0) {
    const parts = normalized.split("/");
    const parentPath = parts.slice(0, -1).join("/");
    if (parentPath) {
      return allEntries.filter((entry) => entry.startsWith(parentPath));
    }
  }

  return matches;
}

function grepText(text: string, pattern: string, context: number): string[] {
  let regex: RegExp;
  try {
    regex = new RegExp(pattern);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Invalid regex pattern: ${message}\n\nPattern: ${pattern}\n\nTip: Escape special characters like ( ) [ ] { } . * + ? ^ $ \\ |`);
  }

  const lines = text.split("\n");
  const matches: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    if (!regex.test(lines[i])) continue;
    const start = Math.max(0, i - context);
    const end = Math.min(lines.length, i + context + 1);
    const block = lines
      .slice(start, end)
      .map((line, idx) => `${start + idx + 1}: ${line}`)
      .join("\n");
    matches.push(block);
  }

  return matches;
}

async function handleList(target: string): Promise<void> {
  const jar = await findJar("sources");
  
  // Try adding wildcard if not present and path looks like a directory
  let patterns = [target];
  if (!target.includes("*") && target.endsWith("/")) {
    patterns.push(`${target}*`);
  } else if (!target.includes("*") && !target.includes(".")) {
    // Might be a directory without trailing slash
    patterns.push(`${target}/*`, `${target}*`);
  }

  for (const pattern of patterns) {
    const { code, stdout } = await runCommand("unzip", ["-l", jar, pattern]);
    if (code === 0) {
      console.log(stdout.trim());
      return;
    }
  }

  // No matches found - provide suggestions
  const suggestions = await findSimilarPaths(jar, target);
  if (suggestions.length > 0) {
    console.error(`No entries matching ${target}\n`);
    console.error("Did you mean one of these?\n");
    
    // Group by directory and show unique directories
    const directories = new Set<string>();
    const files = new Set<string>();
    
    for (const entry of suggestions.slice(0, 20)) {
      if (entry.endsWith(".java") || entry.endsWith(".class")) {
        files.add(entry);
      } else {
        const dirMatch = entry.match(/^(.+?\/)[^/]*$/);
        if (dirMatch) directories.add(dirMatch[1]);
      }
    }

    if (directories.size > 0) {
      console.error("Directories:");
      for (const dir of Array.from(directories).slice(0, 10)) {
        console.error(`  ${dir}`);
      }
    }
    
    if (files.size > 0) {
      console.error("\nFiles:");
      for (const file of Array.from(files).slice(0, 10)) {
        console.error(`  ${file}`);
      }
    }
    
    if (suggestions.length > 20) {
      console.error(`\n... and ${suggestions.length - 20} more`);
    }
  } else {
    console.error(`No entries matching ${target}`);
    console.error("\nTip: Try removing trailing slashes or use wildcards like 'path/*'");
  }
  
  process.exit(1);
}

async function handleCat(file: string): Promise<void> {
  const jar = await findJar("sources");
  try {
    const content = await readEntry(jar, file);
    process.stdout.write(content);
  } catch (err) {
    // Try to find similar files
    const suggestions = await findSimilarPaths(jar, file);
    if (suggestions.length > 0) {
      console.error(`File not found: ${file}\n`);
      console.error("Did you mean one of these?");
      for (const suggestion of suggestions.slice(0, 10)) {
        console.error(`  ${suggestion}`);
      }
      if (suggestions.length > 10) {
        console.error(`\n... and ${suggestions.length - 10} more`);
      }
    } else {
      console.error(`File not found: ${file}`);
    }
    process.exit(1);
  }
}

async function handleGrep(pattern: string, file: string): Promise<void> {
  const jar = await findJar("sources");
  try {
    const content = await readEntry(jar, file);
    const results = grepText(content, pattern, config.context);

    if (results.length === 0) {
      console.log(`No matches for "${pattern}" in ${file}.`);
      return;
    }

    for (const block of results) {
      console.log(block);
      console.log("--");
    }
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err);
    
    // Check if it's a regex error
    if (errorMessage.includes("Invalid regex pattern")) {
      console.error(errorMessage);
      process.exit(1);
    }
    
    // Otherwise, assume it's a file-not-found error
    const suggestions = await findSimilarPaths(jar, file);
    if (suggestions.length > 0) {
      console.error(`File not found: ${file}\n`);
      console.error("Did you mean one of these?");
      for (const suggestion of suggestions.slice(0, 10)) {
        console.error(`  ${suggestion}`);
      }
      if (suggestions.length > 10) {
        console.error(`\n... and ${suggestions.length - 10} more`);
      }
    } else {
      console.error(`File not found: ${file}`);
    }
    process.exit(1);
  }
}

async function handleGrepAll(pattern: string, glob: string): Promise<void> {
  const jar = await findJar("sources");
  
  // Validate regex pattern before processing files
  try {
    new RegExp(pattern);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Invalid regex pattern: ${message}\n\nPattern: ${pattern}\n\nTip: Escape special characters like ( ) [ ] { } . * + ? ^ $ \\ |`);
    process.exit(1);
  }
  
  const entries = await listEntries(jar, glob);
  let printed = false;

  for (const entry of entries) {
    const content = await readEntry(jar, entry);
    const results = grepText(content, pattern, config.context);
    if (results.length === 0) continue;
    printed = true;
    console.log(`# ${entry}`);
    for (const block of results) {
      console.log(block);
      console.log("--");
    }
  }

  if (!printed) {
    console.log(`No matches for "${pattern}" in ${glob}.`);
  }
}

async function handleAsset(file: string): Promise<void> {
  const jar = await findJar("merged");
  for (const entry of [`assets/minecraft/${file}`, file]) {
    const { code, stdout } = await runCommand("unzip", ["-p", jar, entry]);
    if (code === 0) {
      process.stdout.write(stdout);
      return;
    }
  }
  throw new Error(`No asset found for ${file}`);
}

async function handleAssetList(target: string): Promise<void> {
  const jar = await findJar("merged");
  
  // Try multiple variations
  const patterns = [
    target,
    `assets/minecraft/${target}`,
    target.endsWith("/") ? `${target}*` : `${target}/*`,
    `assets/minecraft/${target.endsWith("/") ? `${target}*` : `${target}/*`}`,
  ];

  for (const pattern of patterns) {
    const { code, stdout } = await runCommand("unzip", ["-l", jar, pattern]);
    if (code === 0) {
      console.log(stdout.trim());
      return;
    }
  }

  // No matches - provide suggestions
  const suggestions = await findSimilarPaths(jar, `assets/minecraft/${target}`);
  if (suggestions.length > 0) {
    console.error(`No assets matching ${target}\n`);
    console.error("Did you mean one of these?");
    for (const suggestion of suggestions.slice(0, 10)) {
      // Show path relative to assets/minecraft/ if possible
      const display = suggestion.startsWith("assets/minecraft/")
        ? suggestion.substring("assets/minecraft/".length)
        : suggestion;
      console.error(`  ${display}`);
    }
    if (suggestions.length > 10) {
      console.error(`\n... and ${suggestions.length - 10} more`);
    }
  } else {
    console.error(`No assets matching ${target}`);
  }
  
  process.exit(1);
}

async function main(): Promise<void> {
  if (!command) {
    usage();
    process.exit(1);
    return;
  }

  try {
    switch (command) {
      case "list": {
        const target = parsedArgs[0];
        if (!target) throw new Error("list requires a path");
        await handleList(target);
        break;
      }
      case "cat": {
        const file = parsedArgs[0];
        if (!file) throw new Error("cat requires a file path");
        await handleCat(file);
        break;
      }
      case "grep": {
        const [pattern, file] = parsedArgs;
        if (!pattern || !file)
          throw new Error("grep requires a pattern and file");
        await handleGrep(pattern, file);
        break;
      }
      case "grep-all": {
        const [pattern, glob] = parsedArgs;
        if (!pattern || !glob)
          throw new Error("grep-all requires a pattern and glob");
        await handleGrepAll(pattern, glob);
        break;
      }
      case "asset": {
        const file = parsedArgs[0];
        if (!file) throw new Error("asset requires a file path");
        await handleAsset(file);
        break;
      }
      case "asset-list": {
        const target = parsedArgs[0];
        if (!target) throw new Error("asset-list requires a path");
        await handleAssetList(target);
        break;
      }
      default:
        throw new Error(`Unknown command: ${command}`);
    }
  } catch (err) {
    console.error((err as Error).message);
    process.exit(1);
  }
}

await main();
