<script lang="ts">
import { resolve } from '$app/paths';
import Meta from '$lib/components/Meta.svelte';
import type { ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import ShaderCard from '$lib/components/ShaderCard.svelte';
import CompactRow from '$lib/components/CompactRow.svelte';
import {
	Counter,
	DataView,
	Grid,
	Pagination,
	Toolbar,
	ViewToggle,
	createCursorList
} from '$lib/components/data-view';
import { type ResultOf } from '$lib/graphql';
import BrandIcon from '$lib/components/icons/BrandIcon.svelte';
import { Camera, Eye, Sparkles } from '@lucide/svelte';
import { fly } from 'svelte/transition';
import { untrack } from 'svelte';
import type { PageData } from './$types';
import { AuthorShadersQuery } from './queries';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

const author = $derived(data.author);

const list = createCursorList<ShaderCardShader>({
	key: (s) => s.id,
	initial: () => data.shaders,
	query: AuthorShadersQuery,
	extract: (d: ResultOf<typeof AuthorShadersQuery>) =>
		d.author?.shaders ?? { edges: [], pageInfo: { hasNextPage: false }, totalCount: 0 },
	fixedFilters: { slug: untrack(() => data.author.slug) },
	pageSize: 24
});

const modrinthPlatform = $derived(author.platforms.find((p) => p.platform === 'modrinth'));
const curseforgePlatform = $derived(author.platforms.find((p) => p.platform === 'curseforge'));
</script>

<Meta
	title="{author.name} - Shaders"
	description="{author.name} has {author.shaderCount} shader {author.shaderCount === 1 ? 'pack' : 'packs'} on Glint with {author.totalCaptures} captures."
	image={author.imagePath}
/>

<div class="py-6">
	<!-- Header -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-8">
		<h1 class="text-3xl font-bold tracking-tight">
			{author.name}
		</h1>

		<!-- Stats row -->
		<div class="mt-3 flex flex-wrap items-center gap-4 text-sm text-foreground">
			<span class="inline-flex items-center gap-1.5">
				<Sparkles class="h-4 w-4 opacity-60" />
				{author.shaderCount}
				{author.shaderCount === 1 ? 'shader' : 'shaders'}
			</span>
			<span class="inline-flex items-center gap-1.5">
				<Camera class="h-4 w-4 opacity-60" />
				{author.totalCaptures.toLocaleString()} captures
			</span>
			<span class="inline-flex items-center gap-1.5">
				<Eye class="h-4 w-4 opacity-60" />
				{author.totalViews.toLocaleString()} views
			</span>
		</div>

		<!-- Platform links -->
		<!-- eslint-disable svelte/no-navigation-without-resolve -->
		<!-- eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- boolean OR is intentional in {#if} condition -->
		{#if modrinthPlatform?.url || curseforgePlatform?.url}
			<div class="mt-3 flex items-center gap-2">
				{#if modrinthPlatform?.url}
					<a
						href={modrinthPlatform.url}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-1.5 rounded-md bg-card px-3 py-1.5 text-sm text-card-foreground ring-1 ring-border transition-colors hover:bg-muted"
					>
						<BrandIcon name="modrinth" />
						Modrinth
					</a>
				{/if}
				{#if curseforgePlatform?.url}
					<a
						href={curseforgePlatform.url}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-1.5 rounded-md bg-card px-3 py-1.5 text-sm text-card-foreground ring-1 ring-border transition-colors hover:bg-muted"
					>
						<BrandIcon name="curseforge" />
						CurseForge
					</a>
				{/if}
			</div>
		{/if}
		<!-- eslint-enable svelte/no-navigation-without-resolve -->

		<!-- Top shader highlight -->
		{#if author.topShaderName && author.topShaderSlug}
			<div class="mt-4">
				<a
					href={resolve('/shaders/[slug]', { slug: author.topShaderSlug })}
					class="inline-flex items-center gap-2 rounded-lg bg-card px-4 py-2 ring-1 ring-border transition-colors hover:bg-muted"
				>
					<span class="text-sm text-foreground/70">Most popular:</span>
					<span class="text-sm font-semibold text-card-foreground">{author.topShaderName}</span>
				</a>
			</div>
		{/if}
	</div>

	<!-- Shader Grid -->
	<Toolbar class="mb-6">
		<h2 class="text-lg font-semibold">Shaders</h2>
		<div class="flex-1"></div>
		<ViewToggle bind:mode={list.viewMode} />
	</Toolbar>

	<DataView {list}>
		{#snippet empty()}
			<div class="flex flex-col items-center justify-center py-16 text-center">
				<Sparkles class="mb-4 h-8 w-8 text-foreground opacity-50" strokeWidth={1.5} />
				<h3 class="text-lg font-semibold text-foreground">No shaders yet</h3>
				<p class="mt-1 text-sm text-foreground/70">This author has no shaders with captures</p>
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
				/>
			{/snippet}
		</Grid>

		<Counter noun="shader" />

		<Pagination style="load-more" />
	</DataView>
</div>
