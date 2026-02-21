<script lang="ts">
import { goto, invalidate } from '$app/navigation';
import { page } from '$app/state';
import { api } from '$lib/api';
import type { Shader, ShaderSearchResult, ShaderSearchSort } from '$lib/bindings';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import {
	DataView,
	Grid,
	Pagination,
	Search as SearchInput,
	Sort,
	Table,
	Toolbar,
	ViewToggle,
	createClientList,
	filter
} from '$lib/components/data-view';
import type { FilterDescriptor, ListState, ViewMode } from '$lib/components/data-view';
import { BooleanToggle } from '$lib/components/filters';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import * as Tabs from '$lib/components/ui/tabs';
import { cn } from '$lib/utils';
import { formatNumber } from '$lib/utils/format';
import { withRetry } from '$lib/utils/retry';
import {
	CircleAlert,
	Download,
	ExternalLink,
	EyeOff,
	Flame,
	Link,
	Search,
	Sparkles,
	X as XIcon
} from '@lucide/svelte';
import type { PageData } from './$types';
import type { AdminShader } from './queries';
import { columns } from './columns.js';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let shaders = $derived(data.shaders);
let shadersError = $derived(data.error);

// Tab state (synced to URL)
type PageTab = 'my-shaders' | 'discover';
let activeTab = $state<PageTab>((page.url.searchParams.get('tab') as PageTab) || 'my-shaders');

function handleTabChange(tab: string) {
	activeTab = tab as PageTab;
	const url = new URL(page.url);
	if (tab === 'my-shaders') {
		url.searchParams.delete('tab');
	} else {
		url.searchParams.set('tab', tab);
	}
	void goto(url.toString(), {
		replaceState: true,
		noScroll: true,
		keepFocus: true
	});
}

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions -- needs implicit index signature for Record<string, FilterDescriptor>
type ShaderFilters = {
	captureDisabled: FilterDescriptor<boolean>;
	hasPlatformLink: FilterDescriptor<boolean>;
};

const shaderList = createClientList<AdminShader, ShaderFilters>({
	items: () => shaders,
	search: {
		clientFilter: (shader, q) =>
			shader.name.toLowerCase().includes(q) ||
			shader.slug.toLowerCase().includes(q) ||
			(shader.description?.toLowerCase().includes(q) ?? false)
	},
	sort: {
		options: [
			{ value: 'name', label: 'Name' },
			{ value: 'versions', label: 'Versions' },
			{ value: 'created', label: 'Created' }
		],
		default: 'name',
		compare: (sort) => {
			if (sort === 'name') return (a, b) => a.name.localeCompare(b.name);
			if (sort === 'versions') return (a, b) => b.versionCount - a.versionCount;
			if (sort === 'created')
				return (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
		}
	},
	filters: {
		captureDisabled: filter(false),
		hasPlatformLink: filter(false)
	},
	applyFilters: (shader, filters) => {
		// Boolean OR: when filter is on, bypass the enabled check entirely
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		const captureOk = filters.captureDisabled || shader.captureEnabled;
		// Boolean OR: only apply when filter is on; modrinthId/curseforgeId are strings not booleans
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		const platformOk =
			!filters.hasPlatformLink || shader.modrinthId != null || shader.curseforgeId != null;
		return captureOk && platformOk;
	},
	viewMode: 'table'
});

const PAGE_SIZE = 20;

let discoverLoaded = $state(false);
let discoverResults = $state<ShaderSearchResult[]>([]);
let discoverPage = $state(1);
let totalModrinth = $state(0);
let totalCurseforge = $state<number | undefined>(undefined);
let discoverLoading = $state(false);
let discoverLoadingMore = $state(false);
let discoverError = $state<string | null>(null);

let browseSort = $state<ShaderSearchSort>('popular');
let searchActive = $state(false);
let searchQuery = $state('');
let searchInput = $state('');
let hideAdopted = $state(false);

let adoptDialogOpen = $state(false);
let adoptUrl = $state<string | undefined>(undefined);
let adoptDialogUrlMode = $state(true);

// Reset adopt dialog state when it closes
$effect(() => {
	if (!adoptDialogOpen) {
		adoptUrl = undefined;
		adoptDialogUrlMode = true;
	}
});

let hasMore = $derived.by(() => {
	const total = totalModrinth + (totalCurseforge ?? 0);
	return discoverResults.length < total;
});

let filteredResults = $derived.by(() => {
	if (!hideAdopted) return discoverResults;
	return discoverResults.filter((r) => !r.adopted);
});

/* eslint-disable @typescript-eslint/no-empty-function, @typescript-eslint/no-explicit-any */
const discoverList: ListState<ShaderSearchResult, any> = {
	get items() {
		return filteredResults;
	},
	get loading() {
		return discoverLoading || discoverLoadingMore;
	},
	get error() {
		return discoverError;
	},
	get hasMore() {
		return hasMore && !discoverLoading;
	},
	get totalCount() {
		return totalModrinth + (totalCurseforge ?? 0);
	},
	loadMore,
	reset: clearSearch,
	get search() {
		return '';
	},
	set search(_) {},
	get sort() {
		return '';
	},
	set sort(_) {},
	get sortOptions() {
		return [];
	},
	get filters() {
		return {} as Record<string, never>;
	},
	get viewMode(): ViewMode {
		return 'grid';
	},
	set viewMode(_) {}
};
/* eslint-enable @typescript-eslint/no-empty-function, @typescript-eslint/no-explicit-any */

// Load discover on mount if tab param says so
$effect(() => {
	if (activeTab === 'discover' && !discoverLoaded) {
		void loadBrowse(browseSort);
	}
});

async function loadBrowse(sort: ShaderSearchSort) {
	const isInitial = !discoverLoaded;
	if (isInitial) discoverLoading = true;
	discoverError = null;

	const result = await withRetry(() => api.adopt.search(undefined, 1, PAGE_SIZE, sort));
	if (result.isOk) {
		discoverResults = result.value.results;
		discoverPage = 2;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		discoverError = result.error.message;
		discoverResults = [];
	}
	discoverLoading = false;
	discoverLoaded = true;
}

function handleBrowseSortChange(tab: string) {
	if (searchActive) return;
	browseSort = tab as ShaderSearchSort;
	void loadBrowse(browseSort);
}

async function handleSearch() {
	const q = searchInput.trim();
	if (!q) return;

	discoverLoading = true;
	discoverError = null;
	searchActive = true;
	searchQuery = q;

	const result = await withRetry(() => api.adopt.search(q, 1, PAGE_SIZE));
	if (result.isOk) {
		discoverResults = result.value.results;
		discoverPage = 2;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		discoverError = result.error.message;
		discoverResults = [];
	}
	discoverLoading = false;
}

async function loadMore() {
	discoverLoadingMore = true;
	discoverError = null;

	const sort: ShaderSearchSort | undefined = searchActive ? undefined : browseSort;
	const q = searchActive ? searchQuery : undefined;

	const result = await withRetry(() => api.adopt.search(q, discoverPage, PAGE_SIZE, sort));
	if (result.isOk) {
		// Deduplicate: platform pagination can return the same project at different pages
		// when sort order is unstable (e.g. download count ties shifting between pages)
		const existing = new Set(discoverResults.map((r) => r.platform + ':' + r.platform_id));
		const fresh = result.value.results.filter((r) => !existing.has(r.platform + ':' + r.platform_id));
		discoverResults = [...discoverResults, ...fresh];
		discoverPage += 1;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		discoverError = result.error.message;
	}
	discoverLoadingMore = false;
}

function clearSearch() {
	searchActive = false;
	searchQuery = '';
	searchInput = '';
	void loadBrowse(browseSort);
}

function handleSearchKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter' && !discoverLoading) {
		void handleSearch();
	}
}

function adoptFromResult(result: ShaderSearchResult) {
	adoptUrl = result.platform_url;
	adoptDialogUrlMode = false;
	adoptDialogOpen = true;
}

function handleShaderAdopted(shader: Shader) {
	adoptUrl = undefined;
	// Update discover results optimistically
	discoverResults = discoverResults.map((r) => {
		if (r.platform_url === shader.website_url || r.slug === shader.slug) {
			return { ...r, adopted: { id: shader.id, slug: shader.slug } };
		}
		if (
			(r.platform === 'modrinth' && shader.modrinth_id && r.platform_id === shader.modrinth_id) ||
			(r.platform === 'curseforge' && shader.curseforge_id && r.platform_id === shader.curseforge_id)
		) {
			return { ...r, adopted: { id: shader.id, slug: shader.slug } };
		}
		return r;
	});
	// Reload the Current Shaders list
	void invalidate('glint:shaders');
}
</script>

<svelte:head><title>Shaders - Glint</title></svelte:head>

<div class="space-y-4">
  <Breadcrumb segments={[{ label: "Shaders" }]} />

  <Tabs.Root value={activeTab} onValueChange={handleTabChange}>
    <Tabs.List>
      <Tabs.Trigger value="my-shaders">
        <Sparkles class="mr-1.5 h-4 w-4" />
        Current Shaders
        {#if shaders.length > 0}
          <span class="ml-1.5 text-xs text-muted-foreground"
            >{shaders.length}</span
          >
        {/if}
      </Tabs.Trigger>
      <Tabs.Trigger value="discover">
        <Search class="mr-1.5 h-4 w-4" />
        Discover
      </Tabs.Trigger>
    </Tabs.List>

    <!-- Current Shaders Tab -->
    <Tabs.Content value="my-shaders">
      <div class="space-y-3">
        <Toolbar>
          <SearchInput
            bind:value={shaderList.search}
            placeholder="Search shaders..."
          />
          <BooleanToggle
            label="Show disabled"
            bind:checked={shaderList.filters.captureDisabled}
          />
          <BooleanToggle
            label="Has platform link"
            bind:checked={shaderList.filters.hasPlatformLink}
          />
          <div class="flex-1"></div>
          <Sort options={shaderList.sortOptions} bind:value={shaderList.sort} />
          <ViewToggle
            modes={["table", "grid"]}
            bind:mode={shaderList.viewMode}
          />
        </Toolbar>

        <DataView list={shaderList} errorMessage={shadersError}>
          {#snippet empty()}
            <div class="py-12 text-center">
              <Download
                class="mx-auto mb-3 h-10 w-10 text-muted-foreground opacity-50"
              />
              <p class="text-muted-foreground">No shaders found.</p>
              <p class="mt-1 text-sm text-muted-foreground">
                Switch to the Discover tab to find and adopt shaders.
              </p>
            </div>
          {/snippet}

          <Table
            {columns}
            pageSize={50}
            getRowHref={(shader: AdminShader) =>
              `/admin/shaders/${shader.id}`}
          >
            {#snippet card(shader: AdminShader)}
              <div class="flex gap-3">
                {#if shader.iconUrl}
                  <img
                    src={shader.iconUrl}
                    alt=""
                    class="h-10 w-10 shrink-0 rounded object-cover"
                  />
                {:else}
                  <div
                    class="flex h-10 w-10 shrink-0 items-center justify-center rounded bg-muted"
                  >
                    <Sparkles class="h-5 w-5 text-muted-foreground" />
                  </div>
                {/if}
                <div class="min-w-0 flex-1">
                  <div class="font-medium">{shader.name}</div>
                  <div class="text-xs text-muted-foreground">
                    {shader.slug}
                  </div>
                  {#if shader.description}
                    <div
                      class="mt-1 line-clamp-1 text-xs text-muted-foreground"
                    >
                      {shader.description}
                    </div>
                  {/if}
                </div>
                <div
                  class="shrink-0 text-right text-xs text-muted-foreground"
                >
                  <div>{shader.versionCount} ver.</div>
                </div>
              </div>
            {/snippet}
          </Table>
          <div class="mt-3">
            <Pagination style="pages" />
          </div>
          <Grid size="small" key={(s: AdminShader) => s.id}>
            {#snippet card(shader: AdminShader)}
              <a
                href="/admin/shaders/{shader.id}"
                class="group flex flex-col overflow-hidden rounded-lg border border-border bg-card transition-colors hover:border-foreground/20"
              >
                <div class="flex gap-3 p-4">
                  {#if shader.iconUrl}
                    <img
                      src={shader.iconUrl}
                      alt=""
                      class="h-12 w-12 shrink-0 rounded-lg object-cover"
                    />
                  {:else}
                    <div
                      class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-muted"
                    >
                      <Sparkles
                        class="h-6 w-6 text-muted-foreground opacity-50"
                      />
                    </div>
                  {/if}
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="truncate text-sm font-semibold"
                        >{shader.name}</span
                      >
                      {#if !shader.captureEnabled}
                        <span
                          class="shrink-0 rounded bg-destructive/10 px-1.5 py-0.5 text-xs font-medium text-destructive"
                          >Disabled</span
                        >
                      {/if}
                    </div>
                    <code class="text-xs text-muted-foreground"
                      >{shader.slug}</code
                    >
                    {#if shader.description}
                      <p
                        class="mt-1 line-clamp-2 text-xs text-muted-foreground"
                      >
                        {shader.description}
                      </p>
                    {/if}
                  </div>
                </div>
                <div
                  class="flex items-center justify-between border-t px-4 py-2 text-xs text-muted-foreground"
                >
                  <span
                    >{shader.versionCount} version{shader.versionCount !== 1
                      ? "s"
                      : ""}</span
                  >
                  <TimeAgo timestamp={shader.createdAt} />
                </div>
              </a>
            {/snippet}
          </Grid>
        </DataView>
      </div>
    </Tabs.Content>

    <!-- Discover Tab -->
    <Tabs.Content value="discover">
      <div class="space-y-3">
        <!-- Toolbar: search + sort + filters, single row -->
        <div class="flex items-center gap-2">
          <!-- Search input -->
          <div class="relative min-w-0 flex-1">
            <Search
              class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              placeholder="Search shaders..."
              bind:value={searchInput}
              onkeydown={handleSearchKeydown}
              disabled={discoverLoading && !discoverLoaded}
              class="pl-9"
            />
          </div>

          <!-- Sort buttons (disabled during search) -->
          <div
            class="flex rounded-md border {searchActive ? 'opacity-40' : ''}"
          >
            <Button
              variant={browseSort === "popular" && !searchActive
                ? "default"
                : "ghost"}
              size="sm"
              class={cn(
                "rounded-r-none border-r text-xs",
                (browseSort !== "popular" || searchActive) && "bg-muted",
              )}
              disabled={searchActive}
              onclick={() => handleBrowseSortChange("popular")}
            >
              <Flame class="h-3.5 w-3.5" />
              <span class="hidden sm:inline">Popular</span>
            </Button>
            <Button
              variant={browseSort === "recent" && !searchActive
                ? "default"
                : "ghost"}
              size="sm"
              class={cn(
                "rounded-l-none text-xs",
                (browseSort !== "recent" || searchActive) && "bg-muted",
              )}
              disabled={searchActive}
              onclick={() => handleBrowseSortChange("recent")}
            >
              <Sparkles class="h-3.5 w-3.5" />
              <span class="hidden sm:inline">Recent</span>
            </Button>
          </div>

          <!-- Hide adopted toggle -->
          <Button
            variant={hideAdopted ? "secondary" : "outline"}
            size="sm"
            class="text-xs"
            onclick={() => (hideAdopted = !hideAdopted)}
            title={hideAdopted
              ? "Show adopted shaders"
              : "Hide adopted shaders"}
          >
            <EyeOff class="h-3.5 w-3.5" />
            <span class="hidden sm:inline"
              >{hideAdopted ? "Hidden" : "Hide adopted"}</span
            >
          </Button>

          <!-- Adopt by URL -->
          <AdoptShaderDialog
            bind:open={adoptDialogOpen}
            initialUrl={adoptDialogUrlMode ? undefined : adoptUrl}
            onShaderAdopted={handleShaderAdopted}
          >
            {#snippet trigger({ props })}
              <Button
                variant="outline"
                size="icon-sm"
                title="Adopt by URL"
                {...props}
              >
                <Link class="h-4 w-4" />
              </Button>
            {/snippet}
          </AdoptShaderDialog>
        </div>

        <!-- Search active banner -->
        {#if searchActive}
          <div
            class="flex items-center gap-2 rounded-md border bg-card px-3 py-2 text-sm"
          >
            <Search class="h-4 w-4 shrink-0 text-muted-foreground" />
            <span class="text-muted-foreground">
              Results for "<span class="font-medium text-foreground"
                >{searchQuery}</span
              >"
            </span>
            <button
              onclick={clearSearch}
              class="ml-auto rounded-sm p-0.5 text-muted-foreground hover:bg-muted hover:text-foreground"
            >
              <XIcon class="h-4 w-4" />
            </button>
          </div>
        {/if}

        <!-- Result count -->
        {#if discoverLoaded && !discoverLoading && discoverResults.length > 0}
          <p class="text-xs text-muted-foreground">
            {totalModrinth} Modrinth{totalCurseforge != null
              ? ` · ${totalCurseforge} CurseForge`
              : ""}
            {#if hideAdopted && filteredResults.length < discoverResults.length}
              · Showing {filteredResults.length} of {discoverResults.length}
            {/if}
          </p>
        {/if}

        <!-- Load-more error (shown alongside existing results, not handled by DataView) -->
        {#if discoverError && filteredResults.length > 0}
          <Alert variant="destructive">
            <CircleAlert class="h-4 w-4" />
            <div>{discoverError}</div>
          </Alert>
        {/if}

        <DataView list={discoverList}>
          {#snippet loading()}
            <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {#each Array(6) as _, i (i)}
                <div class="rounded-lg border bg-card p-4">
                  <div class="flex gap-3">
                    <div
                      class="h-12 w-12 shrink-0 animate-pulse rounded-lg bg-muted"
                    ></div>
                    <div class="min-w-0 flex-1 space-y-2">
                      <div class="h-4 w-2/3 animate-pulse rounded bg-muted"></div>
                      <div class="h-3 w-1/3 animate-pulse rounded bg-muted"></div>
                      <div
                        class="h-3 w-full animate-pulse rounded bg-muted"
                      ></div>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/snippet}

          {#snippet empty()}
            <p class="py-8 text-center text-sm text-muted-foreground">
              {#if searchActive}
                No shaders found for "{searchQuery}".
              {:else if hideAdopted && discoverResults.length > 0}
                All loaded shaders are already adopted.
              {:else}
                No shaders available.
              {/if}
            </p>
          {/snippet}

          <Grid mode="grid" size="medium" key={(r: ShaderSearchResult) => r.platform + ':' + r.platform_id}>
            {#snippet card(result: ShaderSearchResult)}
              <div
                class="group relative flex flex-col rounded-lg border bg-card transition-colors hover:border-foreground/20 {result.adopted
                  ? 'border-success/30 bg-success/10'
                  : ''}"
              >
                <div class="flex gap-3 p-4 pb-2">
                  {#if result.icon_url}
                    <img
                      src={result.icon_url}
                      alt=""
                      class="h-12 w-12 shrink-0 rounded-lg object-cover"
                    />
                  {:else}
                    <div
                      class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-muted"
                    >
                      <Download class="h-5 w-5 text-muted-foreground" />
                    </div>
                  {/if}
                  <div class="min-w-0 flex-1">
                    <div class="flex items-start gap-1.5">
                      <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                      <a
                        href={result.platform_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="truncate font-medium leading-tight hover:underline"
                        onclick={(e: MouseEvent) => e.stopPropagation()}
                      >
                        {result.name}
                      </a>
                      <ExternalLink
                        class="mt-0.5 h-3 w-3 shrink-0 text-muted-foreground"
                      />
                    </div>
                    <p class="mt-0.5 truncate text-xs text-muted-foreground">
                      {result.author}
                    </p>
                  </div>
                </div>
                {#if result.description}
                  <p
                    class="line-clamp-2 px-4 text-xs leading-relaxed text-muted-foreground"
                  >
                    {result.description}
                  </p>
                {/if}
                <div
                  class="mt-auto flex items-center justify-between gap-2 px-4 pb-3 pt-2"
                >
                  <div
                    class="flex items-center gap-2 text-xs text-muted-foreground"
                  >
                    <span class="tabular-nums">
                      <Download class="mr-0.5 inline h-3 w-3" />{formatNumber(
                        result.downloads,
                      )}
                    </span>
                    {#if result.updated_at}
                      <span class="text-muted-foreground/60">·</span>
                      <TimeAgo timestamp={result.updated_at} />
                    {/if}
                  </div>
                  {#if result.adopted}
                    <Button
                      size="sm"
                      variant="outline"
                      href="/admin/shaders/{result.adopted.slug}"
                      class="h-7 gap-1 text-xs"
                    >
                      View
                    </Button>
                  {:else}
                    <Button
                      size="sm"
                      variant="default"
                      class="h-7 text-xs"
                      onclick={() => adoptFromResult(result)}
                    >
                      Adopt
                    </Button>
                  {/if}
                </div>
              </div>
            {/snippet}
          </Grid>

          <Pagination style="load-more" />
        </DataView>
      </div>
    </Tabs.Content>
  </Tabs.Root>
</div>
