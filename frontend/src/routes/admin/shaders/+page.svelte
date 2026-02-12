<script lang="ts">
import { goto } from '$app/navigation';
import { page } from '$app/state';
import { api } from '$lib/api';
import type { Shader, ShaderListItem, ShaderSearchResult, ShaderSearchSort } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminPageHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import * as Tabs from '$lib/components/ui/tabs';
import { cn } from '$lib/utils';
import { formatNumber } from '$lib/utils/display';
import {
	Check,
	CircleAlert,
	Clock,
	Download,
	ExternalLink,
	EyeOff,
	Flame,
	Link,
	LoaderCircle,
	Search,
	SkipForward,
	Sparkles,
	X as XIcon
} from '@lucide/svelte';
import type { PageData } from './$types';

// --- My Shaders data (from SSR load) ---
interface Props {
	data: PageData;
}
let { data }: Props = $props();
let shaders = $derived(data.shaders);
let shadersError = $derived(data.error);

// --- Tab state (synced to URL) ---
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
	void goto(url.toString(), { replaceState: true, noScroll: true, keepFocus: true });

	// Lazy-load discover data on first visit
	if (tab === 'discover' && !discoverLoaded) {
		void loadBrowse(browseSort);
	}
}

// --- My Shaders table config ---
const columns = [
	{ id: 'icon', key: 'icon_url', name: '', hideOnMobile: true },
	{ id: 'name', key: 'name', name: 'Name', cardTitle: true },
	{ id: 'description', key: 'description', name: 'Description' },
	{ id: 'sync_status', key: 'last_synced_at', name: 'Sync' },
	{ id: 'versions', key: 'version_count', name: 'Versions' },
	{ id: 'extraction', key: 'extraction_summary', name: 'Extraction' },
	{ id: 'created_at', key: 'created_at', name: 'Created' }
];

function getExtractionRowBorder(shader: ShaderListItem): string {
	const s = shader.extraction_summary;
	if (!s) return '';
	if (s.failed > 0) return 'border-l-2 border-l-destructive';
	if (s.pending > 0) return 'border-l-2 border-l-warning';
	return '';
}

function getSyncStatus(shader: ShaderListItem): { label: string; class: string } {
	const hasLink = !!shader.modrinth_id || !!shader.curseforge_id;
	if (!hasLink) return { label: 'No link', class: 'text-muted-foreground' };
	if (!shader.last_synced_at) return { label: 'Never', class: 'text-warning' };

	const days = (Date.now() - new Date(shader.last_synced_at).getTime()) / (1000 * 60 * 60 * 24);
	if (days > 7) return { label: `${Math.floor(days)}d`, class: 'text-destructive' };
	if (days > 1) return { label: `${Math.floor(days)}d`, class: 'text-warning' };
	if (days * 24 > 1)
		return {
			label: `${Math.floor(days * 24)}h`,
			class: 'text-green-600 dark:text-green-400'
		};
	return { label: 'Now', class: 'text-green-600 dark:text-green-400' };
}

function formatTerseTime(dateStr: string): string {
	const days = (Date.now() - new Date(dateStr).getTime()) / (1000 * 60 * 60 * 24);
	if (days > 365) return `${Math.floor(days / 365)}y`;
	if (days > 30) return `${Math.floor(days / 30)}mo`;
	if (days > 1) return `${Math.floor(days)}d`;
	if (days * 24 > 1) return `${Math.floor(days * 24)}h`;
	return 'Now';
}

// --- Discover state ---
const PAGE_SIZE = 20;

let discoverLoaded = $state(false);
let discoverResults = $state<ShaderSearchResult[]>([]);
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

	const result = await api.adopt.search(undefined, PAGE_SIZE, 0, sort);
	if (result.isOk) {
		discoverResults = result.value.results;
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

	const result = await api.adopt.search(q, PAGE_SIZE, 0);
	if (result.isOk) {
		discoverResults = result.value.results;
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

	const offset = discoverResults.length;
	const sort: ShaderSearchSort | undefined = searchActive ? undefined : browseSort;
	const q = searchActive ? searchQuery : undefined;

	const result = await api.adopt.search(q, PAGE_SIZE, offset, sort);
	if (result.isOk) {
		discoverResults = [...discoverResults, ...result.value.results];
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
	// Reload the My Shaders list
	void goto(page.url, { invalidateAll: true, noScroll: true });
}
</script>

<svelte:head><title>Shaders - Glint</title></svelte:head>

<div class="space-y-4">
	<AdminPageHeader title="Shaders" />

	<Tabs.Root value={activeTab} onValueChange={handleTabChange}>
		<Tabs.List>
			<Tabs.Trigger value="my-shaders">
				<Sparkles class="mr-1.5 h-4 w-4" />
				My Shaders
				{#if shaders.length > 0}
					<span class="ml-1.5 text-xs text-muted-foreground">{shaders.length}</span>
				{/if}
			</Tabs.Trigger>
			<Tabs.Trigger value="discover">
				<Search class="mr-1.5 h-4 w-4" />
				Discover
			</Tabs.Trigger>
		</Tabs.List>

		<!-- My Shaders Tab -->
		<Tabs.Content value="my-shaders">
			{#if shadersError}
				<Alert variant="destructive">Error: {shadersError}</Alert>
			{:else if shaders.length === 0}
				<div class="py-12 text-center">
					<Download class="mx-auto mb-3 h-10 w-10 text-foreground opacity-50" />
					<p class="text-foreground">No shaders yet.</p>
					<p class="mt-1 text-sm text-foreground">
						Switch to the Discover tab to find and adopt shaders.
					</p>
				</div>
			{:else}
				<AdminTable
					data={shaders}
					{columns}
					onRowClick={(shader: ShaderListItem) => goto(`/admin/shaders/${shader.id}`)}
					getRowId={(s: ShaderListItem) => s.id}
				>
				{#snippet cell({ columnId, value, row })}
					{#if columnId === 'icon'}
						{@const shader = row as ShaderListItem}
						{#if shader.icon_url}
							<img
								src={shader.icon_url}
								alt=""
								class="h-8 w-8 shrink-0 rounded object-cover"
							/>
						{:else}
							<div
								class="flex h-8 w-8 shrink-0 items-center justify-center rounded bg-muted"
							>
								<Sparkles class="h-4 w-4 text-muted-foreground" />
							</div>
						{/if}
					{:else if columnId === 'name'}
						{@const shader = row as ShaderListItem}
						<div>
							<span class="font-medium">{value}</span>
							<span class="ml-1.5 font-mono text-xs text-muted-foreground"
								>{shader.slug}</span
							>
						</div>
						{:else if columnId === 'description'}
							{#if value}
								<span class="line-clamp-1 max-w-xs">{value}</span>
							{:else}
								<span class="text-muted-foreground">-</span>
							{/if}
						{:else if columnId === 'sync_status'}
							{@const status = getSyncStatus(row as ShaderListItem)}
							<span class="text-sm font-medium {status.class}"
								>{status.label}</span
							>
					{:else if columnId === 'versions'}
						<span class="text-sm tabular-nums text-muted-foreground">{(row as ShaderListItem).version_count}</span>
					{:else if columnId === 'extraction'}
						{@const summary = (row as ShaderListItem).extraction_summary}
						{#if !summary || summary.total === 0}
							<span class="text-xs text-muted-foreground">No versions</span>
						{:else}
							<div class="flex items-center gap-1">
								{#if summary.completed > 0}
									<Badge variant="default" class="gap-0.5 px-1.5 py-0 text-[10px] bg-green-600 hover:bg-green-600">
										<Check class="h-3 w-3" />{summary.completed}
									</Badge>
								{/if}
								{#if summary.failed > 0}
									<Badge variant="destructive" class="gap-0.5 px-1.5 py-0 text-[10px]">
										<XIcon class="h-3 w-3" />{summary.failed}
									</Badge>
								{/if}
								{#if summary.pending > 0}
									<Badge variant="secondary" class="gap-0.5 px-1.5 py-0 text-[10px]">
										<Clock class="h-3 w-3" />{summary.pending}
									</Badge>
								{/if}
								{#if summary.skipped > 0}
									<Badge variant="outline" class="gap-0.5 px-1.5 py-0 text-[10px]">
										<SkipForward class="h-3 w-3" />{summary.skipped}
									</Badge>
								{/if}
							</div>
						{/if}
					{:else if columnId === 'created_at'}
						{#if value}
							<span class="text-sm tabular-nums text-muted-foreground"
								>{formatTerseTime(value as string)}</span
							>
						{:else}
							<span class="text-muted-foreground">-</span>
						{/if}
					{:else}
						{value ?? '-'}
					{/if}
					{/snippet}
				</AdminTable>
			{/if}
		</Tabs.Content>

		<!-- Discover Tab -->
		<Tabs.Content value="discover">
			<div class="space-y-3">
				<!-- Toolbar: search + sort + filters, single row -->
				<div class="flex items-center gap-2">
					<!-- Search input -->
					<div class="relative min-w-0 flex-1">
						<Search class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
						<Input
							placeholder="Search shaders..."
							bind:value={searchInput}
							onkeydown={handleSearchKeydown}
							disabled={discoverLoading && !discoverLoaded}
							class="pl-9"
						/>
					</div>

					<!-- Sort buttons (disabled during search) -->
					<div class="flex rounded-md border {searchActive ? 'opacity-40' : ''}">
						<Button
							variant={browseSort === 'popular' && !searchActive ? 'default' : 'ghost'}
							size="sm"
							class={cn('rounded-r-none border-r text-xs', (browseSort !== 'popular' || searchActive) && 'bg-muted')}
							disabled={searchActive}
							onclick={() => handleBrowseSortChange('popular')}
						>
							<Flame class="h-3.5 w-3.5" />
							<span class="hidden sm:inline">Popular</span>
						</Button>
						<Button
							variant={browseSort === 'recent' && !searchActive ? 'default' : 'ghost'}
							size="sm"
							class={cn('rounded-l-none text-xs', (browseSort !== 'recent' || searchActive) && 'bg-muted')}
							disabled={searchActive}
							onclick={() => handleBrowseSortChange('recent')}
						>
							<Sparkles class="h-3.5 w-3.5" />
							<span class="hidden sm:inline">Recent</span>
						</Button>
					</div>

					<!-- Hide adopted toggle -->
					<Button
						variant={hideAdopted ? 'secondary' : 'outline'}
						size="sm"
						class="text-xs"
						onclick={() => (hideAdopted = !hideAdopted)}
						title={hideAdopted ? 'Show adopted shaders' : 'Hide adopted shaders'}
					>
						<EyeOff class="h-3.5 w-3.5" />
						<span class="hidden sm:inline">{hideAdopted ? 'Hidden' : 'Hide adopted'}</span>
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
					<p class="text-xs text-foreground">
						{totalModrinth} Modrinth{totalCurseforge != null
							? ` · ${totalCurseforge} CurseForge`
							: ''}
						{#if hideAdopted && filteredResults.length < discoverResults.length}
							· Showing {filteredResults.length} of {discoverResults.length}
						{/if}
					</p>
				{/if}

				<!-- Error -->
				{#if discoverError}
					<Alert variant="destructive">
						<CircleAlert class="h-4 w-4" />
						<div>{discoverError}</div>
					</Alert>
				{/if}

				<!-- Loading skeleton -->
				{#if discoverLoading}
					<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
						{#each Array(6) as _, i (i)}
							<div class="rounded-lg border bg-card p-4">
								<div class="flex gap-3">
									<div
										class="h-12 w-12 shrink-0 animate-pulse rounded-lg bg-muted"
									></div>
									<div class="min-w-0 flex-1 space-y-2">
										<div
											class="h-4 w-2/3 animate-pulse rounded bg-muted"
										></div>
										<div
											class="h-3 w-1/3 animate-pulse rounded bg-muted"
										></div>
										<div class="h-3 w-full animate-pulse rounded bg-muted"></div>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{:else if filteredResults.length === 0 && discoverLoaded && !discoverError}
					<p class="py-8 text-center text-sm text-foreground">
						{#if searchActive}
							No shaders found for "{searchQuery}".
						{:else if hideAdopted && discoverResults.length > 0}
							All loaded shaders are already adopted.
						{:else}
							No shaders available.
						{/if}
					</p>
				{/if}

				<!-- Card grid (separate from empty state so Load More is always reachable) -->
				{#if filteredResults.length > 0}
					<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
						{#each filteredResults as result (result.platform + ':' + result.platform_id)}
							<div
								class="group relative flex flex-col rounded-lg border bg-card transition-colors hover:border-foreground/20 {result.adopted
									? 'border-green-600/30 bg-green-50 dark:border-green-400/40 dark:bg-card'
									: ''}"
							>
								<div class="flex gap-3 p-4 pb-2">
									<!-- Icon -->
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
											<Download
												class="h-5 w-5 text-muted-foreground"
											/>
										</div>
									{/if}

									<!-- Name + author -->
									<div class="min-w-0 flex-1">
										<div class="flex items-start gap-1.5">
											<a
												href={result.platform_url}
												target="_blank"
												rel="noopener noreferrer"
												class="truncate font-medium leading-tight hover:underline"
												onclick={(e: MouseEvent) =>
													e.stopPropagation()}
											>
												{result.name}
											</a>
											<ExternalLink
												class="mt-0.5 h-3 w-3 shrink-0 text-muted-foreground"
											/>
										</div>
										<p
											class="mt-0.5 truncate text-xs text-muted-foreground"
										>
											{result.author}
										</p>
									</div>
								</div>

								<!-- Description -->
								{#if result.description}
									<p
										class="line-clamp-2 px-4 text-xs leading-relaxed text-muted-foreground"
									>
										{result.description}
									</p>
								{/if}

								<!-- Footer: metadata + action -->
								<div
									class="mt-auto flex items-center justify-between gap-2 px-4 pb-3 pt-2"
								>
									<div
										class="flex items-center gap-2 text-xs text-muted-foreground"
									>
										<span class="tabular-nums">
											<Download class="mr-0.5 inline h-3 w-3" />{formatNumber(
												result.downloads
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
						{/each}
					</div>
				{/if}

				<!-- Load more (always visible when there are more results, even if current page is all filtered out) -->
				{#if hasMore && !discoverLoading}
					<div class="flex justify-center pt-2">
						<Button
							variant="outline"
							onclick={loadMore}
							disabled={discoverLoadingMore}
						>
							{#if discoverLoadingMore}
								<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
							{/if}
							Load more
						</Button>
					</div>
				{/if}
			</div>
		</Tabs.Content>
	</Tabs.Root>
</div>


