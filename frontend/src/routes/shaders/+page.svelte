<script lang="ts">
import type { ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import BrowseToolbar from '$lib/components/BrowseToolbar.svelte';
import { CompactRow, ItemGrid, ViewToggle } from '$lib/components/item-grid';
import type { GridMode } from '$lib/components/item-grid';
import Meta from '$lib/components/Meta.svelte';
import ShaderCard from '$lib/components/ShaderCard.svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { createGraphQLClient, query } from '$lib/graphql';
import { AlertTriangle, ChevronRight, Search, X } from '@lucide/svelte';
import { goto, invalidateAll } from '$app/navigation';
import { page } from '$app/stores';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';
import { _BrowseShadersQuery } from './+page';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

// Client-side state
let viewMode = $state<GridMode>('card');
let searchOverride = $state<string | undefined>(undefined);
const searchInput = $derived(searchOverride ?? data.q);
let debounceTimer: ReturnType<typeof setTimeout> | undefined;

// Cursor-based pagination state
let allShaders = $state<ShaderCardShader[]>([]);
let endCursor = $state<string | null>(null);
let hasNextPage = $state(false);
let loadingMore = $state(false);

// Reset accumulated shaders when SSR data changes (search/sort navigation)
$effect(() => {
	allShaders = data.shaders;
	endCursor = data.endCursor;
	hasNextPage = data.hasNextPage;
});

// Clear override when URL-driven data catches up (browser back/forward, navigation)
$effect(() => {
	void data.q;
	searchOverride = undefined;
});

// Clean up debounce timer on unmount
$effect(() => {
	return () => clearTimeout(debounceTimer);
});

// GraphQL client for client-side "load more" fetches
const gqlClient = createGraphQLClient(fetch);

async function loadMore() {
	if (loadingMore || !hasNextPage || !endCursor) return;
	loadingMore = true;
	const result = await query(gqlClient, _BrowseShadersQuery, {
		first: 24,
		after: endCursor,
		search: data.q || undefined,
		sort: data.sort || undefined
	});
	result.match({
		Ok: (d) => {
			/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment -- gql.tada fragment types unresolvable */
			const newShaders = d.shaders.edges.map((e) => e.node as ShaderCardShader);
			const existingIds = new Set(allShaders.map((s) => s.id));
			allShaders = [...allShaders, ...newShaders.filter((s) => !existingIds.has(s.id))];
			/* eslint-enable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment */
			endCursor = d.shaders.pageInfo.endCursor ?? null;
			hasNextPage = d.shaders.pageInfo.hasNextPage;
		},
		Err: (err) => {
			console.warn('Failed to load more shaders:', err.message);
		}
	});
	loadingMore = false;
}

// Navigation helpers
function onSearchInput(value: string) {
	searchOverride = value;
	clearTimeout(debounceTimer);
	debounceTimer = setTimeout(() => {
		const url = new URL($page.url);
		if (value) url.searchParams.set('q', value);
		else url.searchParams.delete('q');
		void goto(url.toString(), { keepFocus: true });
	}, 300);
}

function clearSearch() {
	searchOverride = '';
	clearTimeout(debounceTimer);
	const url = new URL($page.url);
	url.searchParams.delete('q');
	void goto(url.toString(), { keepFocus: true });
}

function setSort(sort: string) {
	const url = new URL($page.url);
	url.searchParams.set('sort', sort);
	void goto(url.toString(), { keepFocus: true });
}

const hasError = $derived(!!data.error);
const hasFilters = $derived(data.q !== '');

// OG image: use the most popular shader's capture image
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access -- gql.tada fragment types
const ogImage = $derived(allShaders[0]?.imagePath ?? null);
</script>

<Meta
	title="Shaders"
	description="Browse and compare Minecraft shaders. See how each shader transforms your game with side-by-side screenshots."
	image={ogImage}
	ogImagePath="/og/shaders/og.png"
/>

<div class="py-6">
	<!-- Title -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-6">
		<h1 class="text-2xl font-semibold tracking-tight">
			Shaders
			{#if !hasError}
				<span class="ml-1 text-lg font-normal text-muted-foreground"
					>({data.total})</span
				>
			{/if}
		</h1>
	</div>

	<!-- Toolbar -->
	<BrowseToolbar class="mb-6">
		<!-- Search -->
		<div class="relative min-w-0 flex-1 sm:min-w-48 sm:flex-initial">
			<Search
				class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
				strokeWidth={2}
			/>
			<Input
				type="text"
				placeholder="Search..."
				value={searchInput}
				oninput={(e: Event) => onSearchInput((e.target as HTMLInputElement).value)}
				disabled={hasError}
				class="w-full pr-8 pl-9 sm:w-48 sm:focus:w-64"
			/>
			{#if hasFilters}
				<button
					class="absolute top-1/2 right-2 -translate-y-1/2 rounded-sm p-0.5 text-muted-foreground hover:text-foreground"
					onclick={clearSearch}
					aria-label="Clear search"
				>
					<X class="h-3.5 w-3.5" />
				</button>
			{/if}
		</div>

		<!-- Sort -->
		<select
			value={data.sort}
			onchange={(e: Event) => setSort((e.target as HTMLSelectElement).value)}
			disabled={hasError}
			class="h-9 rounded-md border border-input bg-background px-3 text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30"
		>
			<option value="popular">Popular</option>
			<option value="updated">Recent</option>
			<option value="name">A-Z</option>
		</select>

		<!-- Spacer pushes view toggle to the right -->
		<div class="flex-1"></div>

		<ViewToggle mode={viewMode} onchange={(m: GridMode) => { viewMode = m; }} />
	</BrowseToolbar>

	<!-- Error State -->
	{#if hasError}
		<div class="flex flex-col items-center justify-center py-16 text-center">
			<div class="mb-6 rounded-full bg-destructive/10 p-4">
				<AlertTriangle class="h-8 w-8 text-destructive" strokeWidth={1.5} />
			</div>
			<h3 class="text-lg font-semibold text-foreground">Failed to load shaders</h3>
			<p class="mt-1 max-w-md text-sm text-foreground/70">
				{data.error ?? 'Something went wrong'}
			</p>
			<Button
				variant="outline"
				class="mt-4"
				onclick={() => void invalidateAll()}
			>
				Try again
			</Button>
		</div>
	{:else}
		<!-- eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-assignment -- gql.tada fragment types unresolvable in Svelte templates -->
		<ItemGrid
			items={allShaders}
			key={(s: ShaderCardShader) => s.id}
			mode={viewMode}
			size="medium"
			empty={{ icon: Search, title: 'No shaders found', message: 'Try adjusting your filters' }}
		>
			{#snippet card(shader: ShaderCardShader)}
				<ShaderCard {shader} />
			{/snippet}
			{#snippet row(shader: ShaderCardShader)}
				<CompactRow
				name={shader.name}
				subtitle={shader.authors[0]?.name ? `by ${shader.authors[0].name}` : undefined}
				image={shader.imagePath ?? undefined}
				thumbhash={shader.thumbhash ?? undefined}
				href={`/shaders/${shader.slug}`}
				>
					{#snippet metadata()}
						{#if shader.categories.length > 0}
							<div class="flex gap-1">
								{#each shader.categories.slice(0, 3) as category (category.id)}
									<span
										class="rounded bg-muted px-1.5 py-0.5 text-[11px] text-muted-foreground"
									>
										{category.name}
									</span>
								{/each}
							</div>
						{:else if shader.latestVersion}
							<span>{shader.latestVersion}</span>
						{/if}
					{/snippet}
					{#snippet trailing()}
						<ChevronRight class="h-4 w-4 text-muted-foreground" />
					{/snippet}
				</CompactRow>
			{/snippet}
		</ItemGrid>

		{#if allShaders.length > 0}
			<p class="mt-2 pl-2 text-xs text-foreground/70">
				Showing {allShaders.length} of {data.total} shaders
			</p>
		{/if}

		{#if hasNextPage}
			<div class="mt-6 flex justify-center">
				<Button
					variant="outline"
					onclick={loadMore}
					disabled={loadingMore}
				>
					{#if loadingMore}
						Loading...
					{:else}
						Load More
					{/if}
				</Button>
			</div>
		{/if}
	{/if}
</div>
