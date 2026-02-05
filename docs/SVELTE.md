# Svelte Style Guide (Frontend)

General principles in [STYLE.md](STYLE.md).

## Architecture

### Route Organization

File-based routing with SvelteKit conventions:

```
src/routes/
├── +layout.svelte       # Root layout (nav, sidebar, theme)
├── +page.svelte          # Home page
├── +error.svelte         # Global error boundary
├── shaders/
│   ├── +page.svelte      # Shader listing
│   ├── +page.ts           # Load function
│   └── [id]/
│       ├── +page.svelte  # Shader detail
│       └── +page.ts
└── admin/                # Admin routes
```

- Load functions in `+page.ts` (universal, not server-side)
- No `+page.server.ts` — all data fetching goes through the API client
- Load functions receive SvelteKit's `fetch` for SSR compatibility

### Data Fetching

```typescript
export const load: PageLoad = async ({ fetch }) => {
    const api = createApiClient(fetch);
    const result = await api.shaders.list();
    return result.match({
        Ok: (shaders) => ({ shaders }),
        Err: () => ({ shaders: [] }),
    });
};
```

- Pass `fetch` from load context to `createApiClient` for SSR
- Handle errors in load functions — return fallback data, don't throw
- Component receives data via `let { data } = $props()`

## Error Handling

- **API layer**: `true-myth` Result pattern with `.match({ Ok, Err })` for exhaustive handling
- **Components**: `<svelte:boundary onerror={handler}>` for render-time errors
- **Load functions**: Catch errors and return fallback data rather than throwing
- Never let API errors bubble unhandled — always match or catch

```svelte
<svelte:boundary onerror={(e) => console.error(e)}>
    <RiskyComponent />
    {#snippet failed(error)}
        <ErrorFallback {error} />
    {/snippet}
</svelte:boundary>
```

Use error boundaries around components that do complex rendering (canvas, dynamic layouts, user-generated content).

## State Management

**Escalation ladder** — use the simplest pattern that works:

1. **Component-local `$state()`** — default for most state
2. **Module-level runes** (`.svelte.ts` files) — when 2+ unrelated components share state
3. **Svelte context** — for deep component trees, avoiding prop drilling
4. **Global stores** — only for truly global concerns (theme, auth, navigation)

```typescript
// 1. Component-local
let count = $state(0);
let doubled = $derived(count * 2);

// 2. Module-level store (theme.svelte.ts)
export const themeStore = createThemeStore();

// 3. Context (in layout)
setContext('captures', captureState);
// (in child)
const captures = getContext<CaptureState>('captures');
```

## Component Patterns

### Granularity

- Extract components when they're **reused**, **too large** (100+ lines of template), or **mature enough** to be a stable abstraction
- Don't extract prematurely — inline code in `+page.svelte` is fine while iterating
- shadcn-svelte components in `$lib/components/ui/` are the base layer; compose from these

### Props

Svelte 5 `$props()` with TypeScript interfaces:

```svelte
<script lang="ts">
    interface Props {
        shader: Shader;
        selected?: boolean;
        onSelect?: (id: number) => void;
    }
    let { shader, selected = false, onSelect }: Props = $props();
</script>
```

- Use function props for callbacks (`onSelect`, `onDelete`), not custom events
- Default optional props in the destructuring
- Use `Snippet` type for render-prop / slot patterns

### Reactivity

- `$state()` for mutable reactive values
- `$derived()` for computed values (replaces `$:` reactive statements)
- `$derived.by()` for complex computations that need a function body
- `$effect()` for side effects (DOM manipulation, event listeners, external subscriptions)
- Avoid `$effect` for data transformations — use `$derived` instead

## External Links

External URLs (Modrinth, CurseForge, etc.) should NOT use `resolve()` — disable the `svelte/no-navigation-without-resolve` rule with `<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->` above the link. Only internal app routes need `resolve()`.

## Styling

- **Tailwind utility classes** directly on elements
- **`cn()` helper** (clsx + tailwind-merge) for conditional class composition
- **tailwind-variants** for component variants (buttons, badges, cards)
- **Dark mode** via `dark:` prefix with class-based strategy
- No CSS modules, no scoped `<style>` blocks except where Tailwind can't reach

## Type Safety

- Import backend types: `import type { Shader } from '$lib/bindings'`
- `type` imports enforced by ESLint — `import type { ... }` not `import { ... }`
- Generated bindings are the source of truth for API shapes — never duplicate them manually
- Use `$types` imports for SvelteKit page data: `import type { PageData } from './$types'`

## API Client

Endpoint classes per resource, aggregated by the client factory:

```typescript
const api = createApiClient(fetch);
const result = await api.shaders.list();    // Result<Shader[], ApiError>
const shader = await api.shaders.get(id);   // Result<Shader, ApiError>
```

- All methods return `Result<T, ApiError>` — never throw
- JSON requests/responses by default
- SSR-compatible via `fetch` parameter injection

## Logging

- Use `console.warn` / `console.error` for issues that need developer attention
- No logging framework — browser devtools are sufficient for a SvelteKit app
- Avoid `console.log` in committed code (use only for temporary debugging)

## Testing

- **Unit tests**: Vitest (`just test web`)
- **E2E tests**: Playwright (`just test web-e2e`)
- Test user-visible behavior, not component internals
- Use Playwright for flows that span multiple pages or involve navigation
- Use Vitest for utility functions, store logic, and API client behavior
