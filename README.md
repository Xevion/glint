# Glint

Automated screenshot catalog for Minecraft shaders. Browse and compare shaders across standardized scenes.

## Stack

- **Frontend**: SvelteKit, TypeScript, Tailwind, shadcn-svelte
- **Backend**: Rust (Axum, SQLx, PostgreSQL)
- **Mod**: Java/Kotlin (Fabric/NeoForge via Architectury, MC 1.21.4)

## Development

```bash
just check          # Type check + lint (frontend + backend)
just check-mod      # Compile mod
just test           # Run all tests
just smoke [platform]  # Integration test (fabric/neoforge)
```

See [`Justfile`](Justfile) for all commands. Licensed under LGPL-3.0.
