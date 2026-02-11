<script lang="ts">
import { api } from '$lib/api';
import type { Shader, ShaderSearchResult, ShaderSearchSort } from '$lib/bindings';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { Alert } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import * as Table from '$lib/components/ui/table';
import * as Tabs from '$lib/components/ui/tabs';
import { cn } from '$lib/utils';
import {
	Check,
	CircleAlert,
	Download,
	ExternalLink,
	Flame,
	LoaderCircle,
	Search,
	Sparkles,
	X
} from '@lucide/svelte';

type BrowseTab = 'popular' | 'recent';

const PAGE_SIZE = 20;

let query = $state('');
let results = $state<ShaderSearchResult[]>([]);
let totalModrinth = $state(0);
let totalCurseforge = $state<number | null>(null);
let loading = $state(false);
let loadingMore = $state(false);
let error = $state<string | null>(null);

let activeTab = $state<BrowseTab>('popular');
let searchActive = $state(false);
let searchQuery = $state('');

let adoptDialogOpen = $state(false);
let adoptUrl = $state<string | undefined>(undefined);

// Track whether there are more results to load
let hasMore = $derived.by(() => {
	const totalPlatformResults = totalModrinth + (totalCurseforge ?? 0);
	return results.length < totalPlatformResults;
});

// Load popular shaders on mount
$effect(() => {
	void loadBrowse('popular');
});

// Reload when tab changes (user clicks tab)
function handleTabChange(tab: string) {
	if (searchActive) return;
	activeTab = tab as BrowseTab;
	void loadBrowse(activeTab);
}

async function loadBrowse(sort: ShaderSearchSort) {
	loading = true;
	error = null;

	const result = await api.adopt.search(undefined, PAGE_SIZE, 0, sort);
	if (result.isOk) {
		results = result.value.results;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		error = result.error.message;
		results = [];
	}
	loading = false;
}

async function handleSearch() {
	const q = query.trim();
	if (!q) return;

	loading = true;
	error = null;
	searchActive = true;
	searchQuery = q;

	const result = await api.adopt.search(q, PAGE_SIZE, 0);
	if (result.isOk) {
		results = result.value.results;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		error = result.error.message;
		results = [];
	}
	loading = false;
}

async function loadMore() {
	loadingMore = true;
	error = null;

	const offset = results.length;
	const sort: ShaderSearchSort | undefined = searchActive ? undefined : activeTab;
	const q = searchActive ? searchQuery : undefined;

	const result = await api.adopt.search(q, PAGE_SIZE, offset, sort);
	if (result.isOk) {
		results = [...results, ...result.value.results];
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		error = result.error.message;
	}
	loadingMore = false;
}

function clearSearch() {
	searchActive = false;
	searchQuery = '';
	query = '';
	void loadBrowse(activeTab);
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter') {
		void handleSearch();
	}
}

function adoptFromResult(result: ShaderSearchResult) {
	adoptUrl = result.platform_url;
	adoptDialogOpen = true;
}

function handleShaderAdopted(shader: Shader) {
	adoptUrl = undefined;
	// Optimistically update the result in-place
	results = results.map((r) => {
		if (r.platform_url === shader.website_url || r.slug === shader.slug) {
			return { ...r, adopted: { id: shader.id, slug: shader.slug } };
		}
		// Match by platform ID
		if (
			(r.platform === 'modrinth' && shader.modrinth_id && r.platform_id === shader.modrinth_id) ||
			(r.platform === 'curseforge' && shader.curseforge_id && r.platform_id === shader.curseforge_id)
		) {
			return { ...r, adopted: { id: shader.id, slug: shader.slug } };
		}
		return r;
	});
}

function formatNumber(n: number): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
	return n.toLocaleString();
}
</script>

<svelte:head><title>Adopt Shader - Glint</title></svelte:head>

<div class="space-y-4">
	<header>
		<h1 class="text-2xl font-semibold">Adopt Shaders</h1>
	<p class="mt-1 text-sm text-foreground">
		Browse or search for shaders on Modrinth and CurseForge to adopt into Glint.
	</p>
	</header>

	<!-- Search input -->
	<div class="flex gap-2">
		<Input
			placeholder="Search for shaders..."
			bind:value={query}
			onkeydown={handleKeydown}
			disabled={loading}
			class="max-w-md"
		/>
		<Button onclick={handleSearch} disabled={loading || !query.trim()}>
			{#if loading && searchActive}
				<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
			{:else}
				<Search class="mr-2 h-4 w-4" />
			{/if}
			Search
		</Button>
	</div>

	<!-- Tabs / Search header -->
	{#if searchActive}
		<div
			class="flex items-center gap-2 rounded-md border bg-muted/50 px-3 py-2 text-sm text-muted-foreground"
		>
			<Search class="h-4 w-4 shrink-0" />
			<span>
				Search results for "<span class="font-medium text-foreground">{searchQuery}</span>"
			</span>
			<button onclick={clearSearch} class="ml-auto rounded-sm p-0.5 hover:bg-muted">
				<X class="h-4 w-4" />
			</button>
		</div>
	{:else}
		<Tabs.Root value={activeTab} onValueChange={handleTabChange}>
			<Tabs.List>
				<Tabs.Trigger value="popular">
					<Flame class="mr-1.5 h-4 w-4" />
					Popular
				</Tabs.Trigger>
				<Tabs.Trigger value="recent">
					<Sparkles class="mr-1.5 h-4 w-4" />
					Recent
				</Tabs.Trigger>
			</Tabs.List>
		</Tabs.Root>
	{/if}

	{#if error}
		<Alert variant="destructive">
			<CircleAlert class="h-4 w-4" />
			<div>{error}</div>
		</Alert>
	{/if}

	{#if loading}
		<!-- Loading skeleton -->
		<div class="space-y-2">
			{#each Array(6) as _, i (i)}
				<div class="flex items-center gap-3 rounded-lg border p-3">
					<div class="h-8 w-8 animate-pulse rounded bg-muted"></div>
					<div class="flex-1 space-y-1.5">
						<div class="h-4 w-1/3 animate-pulse rounded bg-muted"></div>
						<div class="h-3 w-2/3 animate-pulse rounded bg-muted"></div>
					</div>
					<div class="h-8 w-16 animate-pulse rounded bg-muted"></div>
				</div>
			{/each}
		</div>
	{:else if results.length === 0 && !error}
		<p class="py-8 text-center text-foreground">
			{#if searchActive}
				No shaders found for "{searchQuery}".
			{:else}
				No shaders available.
			{/if}
		</p>
	{:else if results.length > 0}
	<div class="text-xs text-foreground">
		{totalModrinth} Modrinth results{totalCurseforge !== null
			? `, ${totalCurseforge} CurseForge results`
			: ''}
	</div>

		<!-- Desktop table -->
		<Table.Root class="hidden border md:block">
			<Table.Header>
				<Table.Row class="bg-muted/50">
					<Table.Head class="w-10 p-3"></Table.Head>
					<Table.Head class="p-3">Name</Table.Head>
					<Table.Head class="p-3">Platform</Table.Head>
					<Table.Head class="p-3">Author</Table.Head>
					<Table.Head class="p-3 text-right">Downloads</Table.Head>
					<Table.Head class="p-3 text-right">Updated</Table.Head>
					<Table.Head class="w-24 p-3 text-right"></Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each results as result (result.platform + ':' + result.platform_id)}
					<Table.Row
						class={cn(
							'transition-colors last:border-b-0 hover:bg-muted/30',
							result.adopted && 'bg-success/10'
						)}
					>
						<Table.Cell class="p-3">
							{#if result.icon_url}
								<img
									src={result.icon_url}
									alt=""
									class="h-8 w-8 rounded object-cover"
								/>
							{:else}
								<div
									class="flex h-8 w-8 items-center justify-center rounded bg-muted"
								>
									<Download class="h-4 w-4 text-muted-foreground" />
								</div>
							{/if}
						</Table.Cell>
						<Table.Cell class="p-3">
							<div class="flex items-center gap-2">
								<a
									href={result.platform_url}
									target="_blank"
									rel="noopener noreferrer"
									class="font-medium hover:underline"
									onclick={(e: MouseEvent) => e.stopPropagation()}
								>
									{result.name}
									<ExternalLink
										class="ml-1 inline h-3 w-3 text-muted-foreground"
									/>
								</a>
								{#if result.adopted}
								<Badge
									variant="secondary"
									class="gap-1 text-success"
								>
									<Check class="h-3 w-3" />
									Adopted
								</Badge>
							{/if}
						</div>
						<div class="line-clamp-1 text-xs text-muted-foreground">
							{result.description}
						</div>
					</Table.Cell>
					<Table.Cell class="p-3">
						<span
							class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium
								{result.platform === 'modrinth'
								? 'bg-success/15 text-success'
								: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'}"
							>
								{result.platform === 'modrinth' ? 'Modrinth' : 'CurseForge'}
							</span>
						</Table.Cell>
						<Table.Cell class="p-3 text-muted-foreground">{result.author}</Table.Cell>
						<Table.Cell class="p-3 text-right tabular-nums">
							{formatNumber(result.downloads)}
						</Table.Cell>
						<Table.Cell class="p-3 text-right text-muted-foreground">
							{#if result.updated_at}
								<TimeAgo timestamp={result.updated_at} />
							{:else}
								<span>-</span>
							{/if}
						</Table.Cell>
						<Table.Cell class="p-3 text-right">
							{#if result.adopted}
								<Button
									size="sm"
									variant="outline"
									href="/shaders/{result.adopted.slug}"
									class="gap-1"
								>
									<ExternalLink class="h-3.5 w-3.5" />
									View
								</Button>
							{:else}
								<Button
									size="sm"
									variant="outline"
									onclick={() => adoptFromResult(result)}
								>
									Adopt
								</Button>
							{/if}
						</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>

		<!-- Mobile cards -->
		<div class="space-y-2 md:hidden">
			{#each results as result (result.platform + ':' + result.platform_id)}
				<div
					class={cn(
						'space-y-2 rounded-lg border p-3',
						result.adopted && 'border-success/40'
					)}
				>
					<div class="flex items-start gap-3">
						{#if result.icon_url}
							<img
								src={result.icon_url}
								alt=""
								class="h-10 w-10 shrink-0 rounded object-cover"
							/>
						{/if}
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2">
								<a
									href={result.platform_url}
									target="_blank"
									rel="noopener noreferrer"
									class="font-medium hover:underline"
								>
									{result.name}
									<ExternalLink
										class="ml-1 inline h-3 w-3 text-muted-foreground"
									/>
								</a>
							{#if result.adopted}
								<Badge
									variant="secondary"
									class="gap-1 text-success"
								>
									<Check class="h-3 w-3" />
									Adopted
								</Badge>
							{/if}
						</div>
						<div class="line-clamp-1 text-xs text-muted-foreground">
							{result.description}
						</div>
					</div>
				</div>
				<div class="flex items-center justify-between text-xs">
					<div class="flex items-center gap-2">
						<span
							class="inline-flex items-center rounded-full px-2 py-0.5 font-medium
								{result.platform === 'modrinth'
								? 'bg-success/15 text-success'
								: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'}"
							>
								{result.platform === 'modrinth' ? 'Modrinth' : 'CurseForge'}
							</span>
							<span class="text-muted-foreground">{result.author}</span>
							<span class="tabular-nums text-muted-foreground">
								{formatNumber(result.downloads)}
							</span>
							{#if result.updated_at}
								<span class="text-muted-foreground">
									<TimeAgo timestamp={result.updated_at} />
								</span>
							{/if}
						</div>
						{#if result.adopted}
							<Button
								size="sm"
								variant="outline"
								href="/shaders/{result.adopted.slug}"
								class="gap-1"
							>
								<ExternalLink class="h-3.5 w-3.5" />
								View
							</Button>
						{:else}
							<Button
								size="sm"
								variant="outline"
								onclick={() => adoptFromResult(result)}
							>
								Adopt
							</Button>
						{/if}
					</div>
				</div>
			{/each}
		</div>

		<!-- Load more -->
		{#if hasMore}
			<div class="flex justify-center pt-2">
				<Button variant="outline" onclick={loadMore} disabled={loadingMore}>
					{#if loadingMore}
						<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					Load more
				</Button>
			</div>
		{/if}
	{/if}
</div>

<AdoptShaderDialog
	bind:open={adoptDialogOpen}
	initialUrl={adoptUrl}
	onShaderAdopted={handleShaderAdopted}
/>
