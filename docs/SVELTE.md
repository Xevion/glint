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

### Valid exports from `+page.ts` / `+layout.ts`

SvelteKit only allows these named exports from route module files:

```
load  prerender  ssr  csr  trailingSlash  config  entries  _anything
```

Exporting any other runtime value causes a **500 at runtime** — ESLint and `svelte-check` do not catch this, so it fails silently until you hit the route. This is a known tooling gap with no current automated lint rule.

**Do not export queries, types, or constants from `+page.ts`:**

```typescript
// ❌ Bad — causes 500 at runtime
export const AdminShaderQuery = graphql(`...`);
export type AdminShaderData = ...;

// ✅ Good — load is the only export
export const load: PageLoad = async ({ fetch }) => { ... };
```

**Extract shared queries and types to a co-located file:**

```
routes/admin/shaders/[id]/
├── +page.ts          ← only `export const load`
├── +page.svelte
├── queries.ts        ← GraphQL queries + derived types
├── schema.ts         ← form schemas (zod)
└── version-columns.ts ← table column definitions
```

```typescript
// queries.ts
export const AdminShaderQuery = graphql(`...`);
export type AdminShaderData = NonNullable<ResultOf<typeof AdminShaderQuery>['adminShader']>;

// +page.ts
import { AdminShaderQuery } from './queries';
export const load: PageLoad = async ({ params, fetch }) => { ... };

// +page.svelte
import type { AdminShaderData } from './queries';
```

`export type` is erased at compile time and will not cause a runtime error, but placing types that depend on query constants in `+page.ts` is fragile — keep them in `queries.ts` alongside the query they're derived from.

## Error Handling

- **API layer**: `true-myth` Result pattern with `.match({ Ok, Err })` for exhaustive handling
- **Components**: `<svelte:boundary onerror={handler}>` for render-time errors
- **Load functions**: Two patterns depending on page type (see below)
- Never let API errors bubble unhandled — always match or catch

### Load Function Error Patterns

**List pages** — return fallback data + optional error message:
```typescript
return result.match({
    Ok: (items) => ({ items }),
    Err: (error) => ({ items: [], error: error.message }),
});
```

**Detail pages** — throw SvelteKit `error()` for the error page:
```typescript
return result.match({
    Ok: (item) => ({ item }),
    Err: (err) => {
        if (err.type === ApiErrorType.NotFound) error(404, { message: 'Not found' });
        error(500, { message: 'Failed to load data' });
    },
});
```

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

## Navigation Patterns

### Use `<a>` tags for user-initiated navigation

Any element whose primary purpose is "click to go to URL X" MUST be a native `<a>` tag with an `href` attribute. This includes cards, table rows, list items, image thumbnails, and any other clickable region that navigates to a page.

`<a>` tags provide:
- **SvelteKit preloading** — data is fetched on hover/touchstart before the user clicks
- **Browser status bar** — shows destination URL on hover
- **Middle-click / right-click** — "Open in new tab" works natively
- **Accessibility** — screen readers announce the element as a link with its destination
- **Keyboard navigation** — Enter follows the link natively (no `onkeydown` handler needed)

Using `goto()` on a `<div>` or `<button>` click handler breaks ALL of these. Never do it for navigational elements.

### When `goto()` IS appropriate

- **Form submission redirects** — navigating after a successful save/delete
- **URL state sync** — updating search params from inputs, dropdowns, or filters
- **Programmatic redirects** — auth guards, error recovery, post-action navigation
- **`invalidateAll()`** — reloading current page data (use this instead of `goto(currentUrl)`)

### Card components with nested interactive elements

When a card is a link but contains other interactive elements (external links, buttons, dropdowns), use the **stretched link** pattern to avoid nested `<a>` tags (which are invalid HTML):

```svelte
<!-- Card wrapper is a plain div -->
<div class="group relative ...">
    <!-- The primary link stretches over the entire card via ::after -->
    <a
        href="/items/{item.slug}"
        class="... after:absolute after:inset-0 after:content-['']"
    >
        {item.name}
    </a>

    <!-- Interactive sub-elements sit above the stretched link -->
    <div class="relative z-10">
        <a href={externalUrl} target="_blank">External</a>
    </div>
</div>
```

### Components that navigate should accept `href`, not `onclick`

Design components that might navigate to accept an `href` prop and render an `<a>` tag. This lets SvelteKit handle preloading automatically.

- **Good:** `CompactRow` accepts `href` and renders `<a>` when provided
- **Good:** `DataTable` accepts `getRowHref` and renders stretched `<a>` links in rows
- **Bad:** A component that accepts `onclick` and calls `goto()` inside — callers can't benefit from preloading

### Reference implementations

- `CompactRow` (`$lib/components/item-grid/compact-row.svelte`) — renders `<a>`, `<button>`, or `<div>` based on props
- `CaptureCard` (`$lib/components/admin/CaptureCard.svelte`) — entire card wrapped in `<a>`
- `DataTable` (`$lib/components/data-table/data-table.svelte`) — `getRowHref` for row-level navigation
- `ShaderCard` (`$lib/components/ShaderCard.svelte`) — stretched-link pattern for cards with nested interactive elements

## Styling

- **Tailwind utility classes** directly on elements
- **`cn()` helper** (clsx + tailwind-merge) for conditional class composition
- **tailwind-variants** for component variants (buttons, badges, cards)
- **Dark mode** via `dark:` prefix with class-based strategy
- No CSS modules, no scoped `<style>` blocks except where Tailwind can't reach

### SVG Icon Opacity

Never apply transparent colors (`text-foreground/50`) directly to SVG icon components (Lucide, etc.). Each `<path>` renders independently, so overlapping strokes compound opacity and create visible hotspots.

Use `opacity-*` on the element instead — this composites the entire SVG as one layer before fading:

```svelte
<!-- Bad: stroke overlap artifacts -->
<Search class="text-muted-foreground/50" />

<!-- Good: composited as one layer -->
<Search class="text-muted-foreground opacity-50" />
```

Transparent text colors on regular HTML elements (spans, paragraphs) are fine — this only affects multi-path SVGs.

### Muted Foreground and Background Contrast

`text-muted-foreground` is designed for text on **opaque** surfaces (`bg-card`, `bg-muted`, `bg-background`). Glint's page background is not plain white — it layers blurred Minecraft wallpaper images beneath a 70% overlay. In light mode, the effective background is a light gray with subtle color, not pure white. `text-muted-foreground` (oklch 0.552 in light mode) against this produces poor contrast.

**Rule:** Only use `text-muted-foreground` inside elements with an opaque background ancestor (`bg-card`, `bg-muted`, etc.). For text floating directly on the page background (breadcrumbs, subtitles, help text, empty states), use `text-foreground` and express visual hierarchy through font size and weight instead.

```svelte
<!-- Bad: washed out against wallpaper background -->
<p class="text-sm text-muted-foreground">Subtitle text here</p>

<!-- Good: readable, hierarchy via text-sm -->
<p class="text-sm text-foreground">Subtitle text here</p>

<!-- OK: inside an opaque card container -->
<div class="bg-card p-4">
    <p class="text-sm text-muted-foreground">Description inside card</p>
</div>
```

Components with opaque backgrounds where `text-muted-foreground` is safe:
- `ShaderCard` / `SceneCard` (use `bg-card`)
- Admin list rows (use `bg-card`)
- Filter/toolbar containers (use `bg-card`)
- Stats pills with `bg-muted`
- Any element inside a `bg-card`, `bg-muted`, or `bg-background` parent

## Type Safety

- Import backend types: `import type { Shader } from '$lib/bindings'`
- `type` imports enforced by ESLint — `import type { ... }` not `import { ... }`
- Generated bindings are the source of truth for API shapes — never duplicate them manually
- Use `$types` imports for SvelteKit page data: `import type { PageData } from './$types'`

## API Client (REST)

Endpoint classes per resource, aggregated by the client factory:

```typescript
const api = createApiClient(fetch);
const result = await api.shaders.list();    // Result<Shader[], ApiError>
const shader = await api.shaders.get(id);   // Result<Shader, ApiError>
```

- All methods return `Result<T, ApiError>` — never throw
- JSON requests/responses by default
- SSR-compatible via `fetch` parameter injection
- Used for admin CRUD, mutations, and mod-facing endpoints

## GraphQL Client

Public data fetching uses GraphQL via urql + gql-tada (`$lib/graphql/`):

```typescript
import { graphql } from '$lib/graphql/tada';
import { createGraphQLClient } from '$lib/graphql/client';

const ShadersQuery = graphql(`query Shaders { shaders { id name slug } }`);
const client = createGraphQLClient(fetch);
const result = await client.query(ShadersQuery, {});
```

- **gql-tada** provides compile-time type safety — query results are fully typed from the schema
- **Schema auto-generated** from backend async-graphql types (`just bindings` or `just check`)
- **Subscriptions** via graphql-ws for real-time updates (e.g., capture progress)
- **Result pattern** — queries are wrapped in `Result<T, ApiError>` matching the REST client
- **When to use**: Prefer GraphQL for public read-heavy pages (shader listings, scene browsing, comparisons). Use REST for admin CRUD, mutations, and mod API endpoints

## Admin Data Tables

Admin list pages use TanStack Table via `$lib/components/data-table/`:

```typescript
// columns.ts — define columns per resource
import { textColumn, timeColumn, imageColumn } from '$lib/components/data-table/columns';

export const columns = [
    imageColumn<Shader>({ accessorKey: 'image_path', header: '' }),
    textColumn<Shader>({ accessorKey: 'name', header: 'Name' }),
    timeColumn<Shader>({ accessorKey: 'updated_at', header: 'Updated' }),
];
```

- Column definitions in co-located `columns.ts` files alongside the page
- Custom cell renderers as separate `.svelte` components (e.g., `shader-name-cell.svelte`)
- `createTable()` composable in `.svelte.ts` for table state management
- Pagination, sorting, and column visibility built in

## Admin Form Composable

Detail pages use `createAdminForm()` from `$lib/components/admin/admin-form.svelte.ts`:

- Reactive field extraction from source entity via `FieldExtractor<T>` functions
- Dirty tracking — only changed fields are sent on save
- Automatic field reset when the source entity changes
- Wraps save actions in `Result<T, ApiError>` with automatic `invalidateAll()` on success

## Admin Navigation

Admin pages use `Breadcrumb` (not the deleted `AdminPageHeader`/`AdminDetailHeader`):

```svelte
<Breadcrumb segments={[
    { label: 'Shaders', href: '/admin/shaders' },
    { label: shader.name },
]} />

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
