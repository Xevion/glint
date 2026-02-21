<script lang="ts">
import { invalidate } from '$app/navigation';
import AuthorCard from '$lib/components/AuthorCard.svelte';
import CompactRow from '$lib/components/CompactRow.svelte';
import Meta from '$lib/components/Meta.svelte';
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
import { Button } from '$lib/components/ui/button';
import { AlertTriangle, ChevronRight, Search as SearchIcon } from '@lucide/svelte';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';
import type { AuthorCardData } from './+page';
import { BrowseAuthorsQuery } from './queries';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

const list = createCursorList<AuthorCardData>({
	key: (a) => a.slug,
	initial: () => data.authors,
	query: BrowseAuthorsQuery,
	extract: (d: ResultOf<typeof BrowseAuthorsQuery>) => d.authors,
	pageSize: 24,
	search: { debounce: 300 },
	sort: {
		options: [
			{ value: 'popular', label: 'Popular' },
			{ value: 'shaders', label: 'Most Shaders' },
			{ value: 'name', label: 'A-Z' }
		],
		default: 'popular'
	},
	syncUrl: true
});

const hasError = $derived(!!data.error);
</script>

<Meta
	title="Authors"
	description="Browse Minecraft shader authors. Discover creators and explore their shader packs."
/>

<div class="py-6">
	<!-- Title -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-6">
		<h1 class="text-2xl font-semibold tracking-tight">
			Authors
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
				<h3 class="text-lg font-semibold text-foreground">Failed to load authors</h3>
				<p class="mt-1 max-w-md text-sm text-foreground/70">
					{message}
				</p>
				<Button
					variant="outline"
					class="mt-4"
					onclick={() => void invalidate('glint:authors')}
				>
					Try again
				</Button>
			</div>
		{/snippet}

		{#snippet empty()}
			<div class="flex flex-col items-center justify-center py-16 text-center">
				<SearchIcon class="mb-4 h-8 w-8 text-foreground opacity-50" strokeWidth={1.5} />
				<h3 class="text-lg font-semibold text-foreground">No authors found</h3>
				<p class="mt-1 text-sm text-foreground/70">Try adjusting your filters</p>
			</div>
		{/snippet}

		<Grid
			mode={list.viewMode}
			size="medium"
		>
			{#snippet card(author: AuthorCardData)}
				<AuthorCard {author} />
			{/snippet}
			{#snippet row(author: AuthorCardData)}
				<CompactRow
					name={author.name}
					subtitle="{author.shaderCount} {author.shaderCount === 1 ? 'shader' : 'shaders'}"
					image={author.imagePath ?? undefined}
					thumbhash={author.thumbhash ?? undefined}
					href={`/authors/${author.slug}`}
				>
					{#snippet trailing()}
						<ChevronRight class="h-4 w-4 text-muted-foreground" />
					{/snippet}
				</CompactRow>
			{/snippet}
		</Grid>

		<Counter noun="author" />

		<Pagination style="load-more" />
	</DataView>
</div>
