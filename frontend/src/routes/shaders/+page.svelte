<script lang="ts">
import { invalidate } from '$app/navigation';
import Meta from '$lib/components/Meta.svelte';
import type { ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import ShaderCard from '$lib/components/ShaderCard.svelte';
import {
	Counter,
	DataView,
	Grid,
	Pagination,
	Search,
	Sort,
	Toolbar,
	ViewToggle,
	createCursorList
} from '$lib/components/data-view';
import { type ResultOf } from '$lib/graphql';
import CompactRow from '$lib/components/CompactRow.svelte';
import { Button } from '$lib/components/ui/button';
import { AlertTriangle, ChevronRight, Search as SearchIcon } from '@lucide/svelte';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';
import { BrowseShadersQuery } from './queries';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

const list = createCursorList<ShaderCardShader>({
	key: (s) => s.id,
	initial: () => data.shaders,
	query: BrowseShadersQuery,
	extract: (d: ResultOf<typeof BrowseShadersQuery>) => d.shaders,
	pageSize: 24,
	search: { debounce: 300 },
	sort: {
		options: [
			{ value: 'popular', label: 'Popular' },
			{ value: 'updated', label: 'Recent' },
			{ value: 'name', label: 'A-Z' }
		],
		default: 'popular'
	},
	syncUrl: true
});

const hasError = $derived(!!data.error);

// OG image: use the most popular shader's capture image
const ogImage = $derived(list.items[0]?.imagePath ?? null);
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
			{#if !hasError && list.totalCount != null}
				<span class="ml-1 text-lg font-normal text-muted-foreground"
					>({list.totalCount})</span
				>
			{/if}
		</h1>
	</div>

	<!-- Toolbar -->
	<Toolbar class="mb-6">
		<Search bind:value={list.search} disabled={hasError} />
		<Sort options={list.sortOptions} bind:value={list.sort} disabled={hasError} />
		<div class="flex-1"></div>
		<ViewToggle bind:mode={list.viewMode} />
	</Toolbar>

	<!-- Content -->
	<DataView {list}>
		{#snippet error(message)}
			<div class="flex flex-col items-center justify-center py-16 text-center">
				<div class="mb-6 rounded-full bg-destructive/10 p-4">
					<AlertTriangle class="h-8 w-8 text-destructive" strokeWidth={1.5} />
				</div>
				<h3 class="text-lg font-semibold text-foreground">Failed to load shaders</h3>
				<p class="mt-1 max-w-md text-sm text-foreground/70">
					{message}
				</p>
				<Button
					variant="outline"
					class="mt-4"
					onclick={() => void invalidate('glint:shaders')}
				>
					Try again
				</Button>
			</div>
		{/snippet}

		{#snippet empty()}
			<div class="flex flex-col items-center justify-center py-16 text-center">
				<SearchIcon class="mb-4 h-8 w-8 text-foreground opacity-50" strokeWidth={1.5} />
				<h3 class="text-lg font-semibold text-foreground">No shaders found</h3>
				<p class="mt-1 text-sm text-foreground/70">Try adjusting your filters</p>
			</div>
		{/snippet}

		<Grid
			mode={list.viewMode}
			size="medium"
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
		</Grid>

		<Counter noun="shader" />

		<Pagination style="load-more" />
	</DataView>
</div>
