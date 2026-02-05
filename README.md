# Glint

[![CI][ci-badge]][ci-link] [![License][license-badge]][license-link] [![Rust][rust-badge]][rust-link] [![SvelteKit][svelte-badge]][svelte-link] [![Kotlin][kotlin-badge]][kotlin-link]

Automated screenshot catalog for Minecraft shaders. Browse and compare shaders across standardized scenes.

> **Alpha** — under active development with zero external users. Expect breaking changes.

<!-- TODO: Add screenshot/preview image here -->
<!-- ![Glint Preview](docs/preview.png) -->

## Stack

- **Frontend** — SvelteKit + TypeScript + Tailwind
- **Backend** — Rust with Axum and PostgreSQL
- **Mod** — Kotlin/Java on Fabric & NeoForge (MC 1.21.4)

## Development

```bash
just check              # Validate all code (auto-fixes formatting)
just test               # Run all tests in parallel
just smoke [platform]   # Integration test (fabric/neoforge)
just dev -f             # Frontend dev server
just dev -b             # Backend dev server
```

See [`Justfile`](Justfile) for all commands. Licensed under [LGPL-3.0](LICENSE).

<!-- Badges -->
[ci-badge]: https://img.shields.io/github/actions/workflow/status/Xevion/glint/ci.yml?branch=master&style=flat&label=CI
[ci-link]: https://github.com/Xevion/glint/actions/workflows/ci.yml
[license-badge]: https://img.shields.io/badge/license-LGPL--3.0-blue?style=flat
[license-link]: LICENSE
[rust-badge]: https://img.shields.io/badge/Rust-CE422B?style=flat&logo=rust&logoColor=white
[rust-link]: https://www.rust-lang.org/
[svelte-badge]: https://img.shields.io/badge/SvelteKit-FF3E00?style=flat&logo=svelte&logoColor=white
[svelte-link]: https://svelte.dev/
[kotlin-badge]: https://img.shields.io/badge/Kotlin-7F52FF?style=flat&logo=kotlin&logoColor=white
[kotlin-link]: https://kotlinlang.org/
