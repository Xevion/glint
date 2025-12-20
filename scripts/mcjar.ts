#!/usr/bin/env bun

import { spawn } from "bun";
import * as fs from "fs/promises";
import path from "path";

type JarKind = "sources" | "merged";

const SEARCH_ROOT = "mod/.gradle/loom-cache/minecraftMaven/net/minecraft";

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

  for await (const match of glob.scan(SEARCH_ROOT)) {
    const resolved = path.isAbsolute(match)
      ? match
      : path.join(SEARCH_ROOT, match);
    candidates.push(resolved);
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

function grepText(text: string, pattern: string, context: number): string[] {
  const regex = new RegExp(pattern);
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
  const { code, stdout, stderr } = await runCommand("unzip", [
    "-l",
    jar,
    target,
  ]);
  if (code !== 0) {
    throw new Error(stderr || `No entries matching ${target}`);
  }
  console.log(stdout.trim());
}

async function handleCat(file: string): Promise<void> {
  const jar = await findJar("sources");
  const content = await readEntry(jar, file);
  process.stdout.write(content);
}

async function handleGrep(pattern: string, file: string): Promise<void> {
  const jar = await findJar("sources");
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
}

async function handleGrepAll(pattern: string, glob: string): Promise<void> {
  const jar = await findJar("sources");
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
  for (const entry of [target, `assets/minecraft/${target}`]) {
    const { code, stdout } = await runCommand("unzip", ["-l", jar, entry]);
    if (code === 0) {
      console.log(stdout.trim());
      return;
    }
  }
  throw new Error(`No assets matching ${target}`);
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
