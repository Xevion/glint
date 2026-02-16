<script lang="ts">
import { api } from '$lib/api';
import { useRetry } from '$lib/api/retry.svelte';
import type { ShaderListItem } from '$lib/bindings';
import BrowseToolbar from '$lib/components/BrowseToolbar.svelte';
import { CompactRow, ItemGrid, ViewToggle } from '$lib/components/item-grid';
import type { GridMode } from '$lib/components/item-grid';
import Meta from '$lib/components/Meta.svelte';
import ShaderCard from '$lib/components/ShaderCard.svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { AlertTriangle, ChevronRight, LoaderCircle, Search } from '@lucide/svelte';
import { untrack } from 'svelte';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

const shaderRetry = untrack(() =>
	useRetry(() => api.shaders.list(), {
		initial: data.error
			? undefined
			: { items: data.shaders, total: data.total, page: 1, page_size: data.total }
	})
);

const shaders = $derived(shaderRetry.data?.items ?? []);
const loadError = $derived(data.error);
const hasError = $derived(!!shaderRetry.error || (!!loadError && !shaderRetry.data));

// Filter state
let searchQuery = $state('');
let sortBy = $state<'popular' | 'name' | 'updated'>('popular');
let viewMode = $state<GridMode>('card');

const filteredShaders = $derived.by(() => {
	let result = shaders.filter((shader: (typeof shaders)[0]) => {
		if (searchQuery && !shader.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
		return true;
	});

	// Sort
	result = [...result].sort((a, b) => {
		switch (sortBy) {
			case 'popular': {
				// Primary: view count, secondary: upstream downloads
				const viewDiff = b.view_count - a.view_count;
				if (viewDiff !== 0) return viewDiff;
				return (b.upstream_downloads ?? 0) - (a.upstream_downloads ?? 0);
			}
			case 'name':
				return a.name.localeCompare(b.name);
			case 'updated':
				return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
			default:
				return 0;
		}
	});

	return result;
});

const hasFilters = $derived(searchQuery !== '');

// OG image: use the most popular shader's capture image
const ogImage = $derived(shaders[0]?.image_url ?? null);
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
					>({filteredShaders.length})</span
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
				bind:value={searchQuery}
				disabled={hasError}
				class="w-full pr-3 pl-9 sm:w-48 sm:focus:w-64"
			/>
		</div>

		<!-- Sort -->
		<select
			bind:value={sortBy}
			disabled={hasError}
			class="h-9 rounded-md border border-input bg-transparent px-3 text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50"
		>
			<option value="popular">Popular</option>
			<option value="updated">Recent</option>
			<option value="name">A-Z</option>
		</select>

		{#if hasFilters}
			<Button
				variant="ghost"
				size="sm"
				onclick={() => {
					searchQuery = '';
				}}
			>
				Clear
			</Button>
		{/if}

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
				{shaderRetry.error?.message ?? loadError ?? 'Something went wrong'}
			</p>
			<Button
				variant="outline"
				class="mt-4"
				disabled={shaderRetry.loading}
				onclick={() => shaderRetry.retry()}
			>
				{#if shaderRetry.loading}
					<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
					Retrying...
				{:else}
					Try again
				{/if}
			</Button>
		</div>
	{:else}
		<ItemGrid
			items={filteredShaders}
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
					image={shader.image_url}
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
	{/if}
</div>
