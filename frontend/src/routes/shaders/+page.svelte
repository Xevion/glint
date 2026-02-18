<script lang="ts">
import type { ShaderListItem } from '$lib/bindings';
import BrowseToolbar from '$lib/components/BrowseToolbar.svelte';
import { CompactRow, ItemGrid, ViewToggle } from '$lib/components/item-grid';
import type { GridMode } from '$lib/components/item-grid';
import Meta from '$lib/components/Meta.svelte';
import Pagination from '$lib/components/Pagination.svelte';
import ShaderCard from '$lib/components/ShaderCard.svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { AlertTriangle, ChevronRight, Search, X } from '@lucide/svelte';
import { goto, invalidateAll } from '$app/navigation';
import { page } from '$app/stores';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

// Client-side state
let viewMode = $state<GridMode>('card');
let searchOverride = $state<string | undefined>(undefined);
const searchInput = $derived(searchOverride ?? data.q);
let debounceTimer: ReturnType<typeof setTimeout> | undefined;

// Clear override when URL-driven data catches up (browser back/forward, navigation)
$effect(() => {
	void data.q;
	searchOverride = undefined;
});

// Clean up debounce timer on unmount
$effect(() => {
	return () => clearTimeout(debounceTimer);
});

// Navigation helpers
function onSearchInput(value: string) {
	searchOverride = value;
	clearTimeout(debounceTimer);
	debounceTimer = setTimeout(() => {
		const url = new URL($page.url);
		if (value) url.searchParams.set('q', value);
		else url.searchParams.delete('q');
		url.searchParams.set('page', '1');
		void goto(url.toString(), { keepFocus: true });
	}, 300);
}

function clearSearch() {
	searchOverride = '';
	clearTimeout(debounceTimer);
	const url = new URL($page.url);
	url.searchParams.delete('q');
	url.searchParams.set('page', '1');
	void goto(url.toString(), { keepFocus: true });
}

function navigateToPage(p: number) {
	const url = new URL($page.url);
	url.searchParams.set('page', String(p));
	void goto(url.toString(), { keepFocus: true });
}

function setSort(sort: string) {
	const url = new URL($page.url);
	url.searchParams.set('sort', sort);
	url.searchParams.set('page', '1');
	void goto(url.toString(), { keepFocus: true });
}

const hasError = $derived(!!data.error);
const hasFilters = $derived(data.q !== '');

// OG image: use the most popular shader's capture image
const ogImage = $derived(data.shaders[0]?.image_path ?? null);
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
		<ItemGrid
			items={data.shaders}
			key={(s: ShaderListItem) => s.id}
			mode={viewMode}
			size="medium"
			empty={{ icon: Search, title: 'No shaders found', message: 'Try adjusting your filters' }}
		>
			{#snippet card(shader: ShaderListItem)}
				<ShaderCard {shader} />
			{/snippet}
			{#snippet row(shader: ShaderListItem)}
				<CompactRow
				name={shader.name}
				subtitle={shader.authors[0]?.name ? `by ${shader.authors[0].name}` : undefined}
				image={shader.image_path}
				thumbhash={shader.thumbhash}
				href={`/shaders/${shader.slug}`}
				>
					{#snippet metadata()}
						{#if shader.categories.length > 0}
							<div class="flex gap-1">
								{#each shader.categories.slice(0, 3) as category (category)}
									<span
										class="rounded bg-muted px-1.5 py-0.5 text-[11px] text-muted-foreground"
									>
										{category}
									</span>
								{/each}
							</div>
						{:else if shader.latest_version}
							<span>{shader.latest_version}</span>
						{/if}
					{/snippet}
					{#snippet trailing()}
						<ChevronRight class="h-4 w-4 text-muted-foreground" />
					{/snippet}
				</CompactRow>
			{/snippet}
		</ItemGrid>

		<Pagination
			totalCount={data.total}
			page={data.page}
			pageSize={data.pageSize}
			onPageChange={(p: number) => navigateToPage(p)}
		/>
	{/if}
</div>
