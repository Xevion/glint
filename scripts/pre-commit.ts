#!/usr/bin/env bun

import { $ } from "bun";
import { exit } from "process";

async function main() {
  // Get list of staged files
  const stagedResult = await $`git diff --cached --name-only --diff-filter=ACMR`.quiet();
  const stagedFiles = new Set(stagedResult.text().trim().split("\n").filter(Boolean));

  if (stagedFiles.size === 0) {
    exit(0); // Nothing staged, nothing to do
  }

  // Get list of partially staged files (files with both staged and unstaged changes)
  const partiallyStaged = new Set<string>();
  for (const file of stagedFiles) {
    const diffResult = await $`git diff --name-only ${file}`.nothrow().quiet();
    if (diffResult.exitCode === 0 && diffResult.text().trim()) {
      partiallyStaged.add(file);
    }
  }

  // Categorize staged files by subsystem
  const frontendFiles = Array.from(stagedFiles).filter((f) => f.startsWith("frontend/"));
  const backendFiles = Array.from(stagedFiles).filter((f) => f.startsWith("backend/"));
  const agentFiles = Array.from(stagedFiles).filter((f) => f.startsWith("agent/"));
  const modFiles = Array.from(stagedFiles).filter((f) => f.startsWith("mod/"));

  // Track which subsystems need formatting
  let needsFormatting = false;

  // Run format checks for each subsystem with staged files
  if (frontendFiles.length > 0) {
    const checkResult = await $`bun run --cwd frontend format:check`.nothrow().quiet();
    if (checkResult.exitCode !== 0) {
      needsFormatting = true;
    }
  }

  if (backendFiles.length > 0 || agentFiles.length > 0) {
    const backendCheckResult =
      backendFiles.length > 0
        ? await $`cargo fmt --manifest-path backend/Cargo.toml -- --check`.nothrow().quiet()
        : { exitCode: 0 };

    const agentCheckResult =
      agentFiles.length > 0
        ? await $`cargo fmt --manifest-path agent/Cargo.toml -- --check`.nothrow().quiet()
        : { exitCode: 0 };

    if (backendCheckResult.exitCode !== 0 || agentCheckResult.exitCode !== 0) {
      needsFormatting = true;
    }
  }

  if (modFiles.length > 0) {
    const checkResult =
      await $`cd mod && ./gradlew spotlessCheck ktlintCheck --quiet`.nothrow().quiet();
    if (checkResult.exitCode !== 0) {
      needsFormatting = true;
    }
  }

  // If all format checks passed, exit early
  if (!needsFormatting) {
    exit(0);
  }

  console.log("⚠  Formatting issues detected, running auto-format...");

  // Run auto-format for subsystems that need it
  if (frontendFiles.length > 0) {
    await $`bun run --cwd frontend format`.quiet();
  }

  if (backendFiles.length > 0) {
    await $`cargo fmt --manifest-path backend/Cargo.toml`.quiet();
  }

  if (agentFiles.length > 0) {
    await $`cargo fmt --manifest-path agent/Cargo.toml`.quiet();
  }

  if (modFiles.length > 0) {
    await $`cd mod && ./gradlew spotlessApply ktlintFormat --quiet`.quiet();
  }

  // Get files modified by formatting (unstaged changes after formatting)
  const diffResult = await $`git diff --name-only`.quiet();
  const modifiedFiles = diffResult.text().trim().split("\n").filter(Boolean);

  // Check for conflicts with partially staged files
  const conflicts = modifiedFiles.filter((f) => partiallyStaged.has(f));

  if (conflicts.length > 0) {
    console.error("\n❌ ERROR: Cannot auto-stage formatting changes for partially staged files:");
    for (const file of conflicts) {
      console.error(`  - ${file}`);
    }
    console.error("\nPlease either:");
    console.error("  1. Stage all changes: git add <files>");
    console.error("  2. Stash unstaged changes: git stash push --keep-index");
    exit(1);
  }

  // Re-stage formatted files (only those that were originally staged)
  const toStage = modifiedFiles.filter((f) => stagedFiles.has(f));

  if (toStage.length > 0) {
    await $`git add ${toStage}`.quiet();
    console.log(`✓ Auto-formatted and re-staged ${toStage.length} file(s)`);
  }

  exit(0);
}

main();
