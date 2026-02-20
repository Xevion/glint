<script lang="ts">
import { resolve } from '$app/paths';
import type { ShaderCardShader } from '$lib/components/ShaderCard.svelte';

import CaptureBadges from '$lib/components/CaptureBadges.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { CompactRow, ItemGrid, MiniCard } from '$lib/components/item-grid';
import Lightbox from '$lib/components/Lightbox.svelte';
import Meta from '$lib/components/Meta.svelte';
import SectionBoundary from '$lib/components/SectionBoundary.svelte';
import BrandIcon from '$lib/components/icons/BrandIcon.svelte';
import * as Collapsible from '$lib/components/ui/collapsible';
import * as Select from '$lib/components/ui/select';
import {
	formatDate,
	formatNumber,
	formatVersion,
	getCurseforgeUrl,
	getModrinthUrl
} from '$lib/utils/format';
import { imageUrl } from '$lib/utils/image';
import { themeStore } from '$lib/stores/theme.svelte';
import { createGraphQLClient, query } from '$lib/graphql';
import {
	Camera,
	ChevronDown,
	ChevronRight,
	Code,
	Download,
	Eye,
	GitCompareArrows,
	Globe,
	ImageOff,
	Layers,
	Pin
} from '@lucide/svelte';
import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';
import {
	type ShaderDetail,
	type ShaderDetailCapture,
	_ShaderDetailQuery,
	_toShaderDetail
} from './+page.ts';

interface Props {
	data: PageData;
}

let { data }: Props = $props();

// Local override: set when version changes, cleared on navigation
let shaderOverride = $state<ShaderDetail | null>(null);

// Pagination state for captures (client-side slicing from full array)
let allCaptures = $state<ShaderDetailCapture[]>([]);
let visibleCount = $state(0);
const capturesPageSize = 24;
const hasMoreCaptures = $derived(visibleCount < allCaptures.length);

// Initialize/reset captures from SSR data when shader page data changes.
// The full capture array lives on data.shader.captures (GraphQL returns all);
// capturesData.items is just the initial page slice from SSR.
$effect(() => {
	const fullCaptures = data.shader.captures;
	allCaptures = fullCaptures;
	visibleCount = data.capturesData.pageSize;
});

// Reset user overrides when navigating between shader pages
$effect(() => {
	void data.shader;
	shaderOverride = null;
	versionOverride = null;
	_selectedCaptureId = null;
	selectedProfileId = null;
	iconErrored = false;
});

// Core data: prefer override (from version change), fall back to page data
const shader = $derived(shaderOverride ?? data.shader);
let captures = $derived(allCaptures.slice(0, visibleCount));

// Version selection
const versions = $derived(shader.versions);
const defaultVersionId = $derived.by(() => {
	// Prefer pinned version if set
	if (shader.preferredVersionId) {
		const pinned = versions.find((v) => v.id === shader.preferredVersionId);
		if (pinned) return pinned.id;
	}
	// Fall back to first version with captures, then first version overall
	return versions.find((v) => v.captureCount > 0)?.id ?? versions[0]?.id ?? null;
});
let versionOverride = $state<string | null>(null);
const selectedVersionId = $derived(versionOverride ?? defaultVersionId);
const selectedVersion = $derived(
	versions.find((v) => v.id === selectedVersionId) ?? versions[0] ?? null
);

// Profiles from extraction data for the selected version.
// Filter out any with empty displayName — these would render as invisible buttons.
const extractedProfiles = $derived.by(() => {
	const raw = shader.profiles ?? [];
	const valid = raw.filter((p) => p.displayName.length > 0);
	if (valid.length < raw.length) {
		console.warn(
			`[profiles] ${raw.length - valid.length} profile(s) have empty displayName and were hidden`,
			raw.filter((p) => p.displayName.length === 0)
		);
	}
	return valid;
});

// Header metadata derivations
const allFeatures = $derived.by(() => {
	const m = shader.metadata;
	if (!m) return [];
	const features: string[] = [];
	if (m.irisFeaturesRequired) features.push(...m.irisFeaturesRequired);
	if (m.irisFeaturesOptional) features.push(...m.irisFeaturesOptional);
	return features;
});
const modrinthUrl = $derived(getModrinthUrl(shader.modrinthId));
const curseforgeUrl = $derived(getCurseforgeUrl(shader.curseforgeId));
const downloadLink = $derived.by(() => {
	if (modrinthUrl) return { url: modrinthUrl, platform: 'modrinth' as const, color: '#00af5c' };
	if (curseforgeUrl)
		return { url: curseforgeUrl, platform: 'curseforge' as const, color: '#f16436' };
	return null;
});
const hasLinks = $derived(!!shader.websiteUrl || !!shader.sourceUrl);

function formatFeatureName(feature: string): string {
	return feature.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
}

// Active profile filter (null = show all captures)
let selectedProfileId = $state<string | null>(null);

// Lightbox state
let _selectedCaptureId = $state<string | null>(null);
let lightboxOpen = $state(false);
let lightboxIndex = $state(0);
let iconErrored = $state(false);

// Captures already match Lightbox's CaptureItem interface (camelCase)
const lightboxCaptures = $derived(captures);

// Guards stale responses from racing version/profile fetches
let fetchGeneration = 0;
const gqlClient = createGraphQLClient(fetch);

/** Re-fetch shader detail via GraphQL when version or profile changes. */
async function refetchShader(versionId?: string, profileId?: string) {
	const generation = ++fetchGeneration;

	const result = await query(gqlClient, _ShaderDetailQuery, {
		id: shader.slug,
		versionId: versionId ?? null,
		profileId: profileId ?? null
	});
	if (generation !== fetchGeneration) return;

	result.match({
		Ok: (responseData) => {
			if (!responseData.shader) return;
			const updated = _toShaderDetail(responseData.shader);
			shaderOverride = updated;
			allCaptures = updated.captures;
			visibleCount = capturesPageSize;
		},
		Err: (err) => {
			console.warn('Failed to fetch shader detail:', err.message);
		}
	});
}

async function onVersionChange(versionId: string) {
	versionOverride = versionId;
	_selectedCaptureId = null;
	selectedProfileId = null;
	await refetchShader(versionId);
}

async function onProfileChange(profileId: string | null) {
	selectedProfileId = profileId;
	_selectedCaptureId = null;
	await refetchShader(selectedVersionId ?? undefined, profileId ?? undefined);
}

function loadMoreCaptures() {
	if (!hasMoreCaptures) return;
	visibleCount = Math.min(visibleCount + capturesPageSize, allCaptures.length);
}

function openLightbox(captureIndex: number) {
	lightboxIndex = captureIndex;
	lightboxOpen = true;
}

// Svelte action for infinite scroll IntersectionObserver
function observeSentinel(node: HTMLElement) {
	const observer = new IntersectionObserver(
		(entries) => {
			if (entries[0]?.isIntersecting) void loadMoreCaptures();
		},
		{ rootMargin: '200px' }
	);
	observer.observe(node);
	return { destroy: () => observer.disconnect() };
}

// OG metadata
const firstCapture = $derived(captures[0] ?? null);
const ogImage = $derived(firstCapture?.imagePath ?? null);
const ogDescription = $derived.by(() => {
	const parts = [`${shader.name} shader for Minecraft`];
	if (selectedVersion) parts.push(`v${selectedVersion.version}`);
	if (allCaptures.length > 0)
		parts.push(`${allCaptures.length} screenshot${allCaptures.length === 1 ? '' : 's'}`);
	return parts.join(' \u00b7 ');
});
</script>

<Meta
	title={shader.name}
	description={ogDescription}
	image={ogImage}
	ogImagePath={`/og/shader/${shader.slug}/og.png`}
/>

{#key shader.id}
	<div class="py-8">
		<!-- Breadcrumb -->
		<nav
			aria-label="Breadcrumb"
			in:fly|local={{ y: -10, duration: 400 }}
			class="mb-6 flex items-center gap-2 text-sm text-foreground"
		>
			<a href={resolve('/shaders', {})} class="transition-colors hover:text-foreground">
				Shaders
			</a>
			<ChevronRight class="h-4 w-4" strokeWidth={2} />
			<span class="font-medium text-foreground">{shader.name}</span>
		</nav>

		<!-- Unified header card -->
		<div
			in:fly|local={{ y: 10, duration: 400, delay: 100 }}
			class="mb-8 rounded-xl border border-border bg-card p-3 sm:p-5"
		>
			<!-- Top row: icon + title/author + action buttons -->
			<div class="flex flex-wrap items-start gap-x-3 gap-y-2 sm:gap-x-4">
				{#if shader.iconUrl && !iconErrored}
					<img
						src={shader.iconUrl}
						alt="{shader.name} icon"
						class="h-14 w-14 shrink-0 rounded-lg object-cover"
						onerror={() => (iconErrored = true)}
					/>
				{/if}
				<div class="min-w-0 flex-1 basis-44">
					<div class="flex flex-wrap items-baseline gap-x-2">
						<h1 class="text-2xl font-bold text-card-foreground sm:text-3xl">{shader.name}</h1>
						{#if shader.authors.length > 0}
							<!-- eslint-disable svelte/no-navigation-without-resolve -->
							<span class="text-sm text-muted-foreground">
								by {#each shader.authors as author, i (author.name)}{#if i > 0},
								{/if}{#if author.url}<a
											href={author.url}
											target="_blank"
											rel="noopener noreferrer"
											class="text-card-foreground underline decoration-foreground/30 underline-offset-2 transition-colors hover:decoration-foreground"
										>{author.name}</a
									>{:else}<span class="text-card-foreground">{author.name}</span>{/if}{/each}
							</span>
							<!-- eslint-enable svelte/no-navigation-without-resolve -->
						{/if}
				</div>
			</div>

			<!-- Action buttons (top-right) -->
				<!-- eslint-disable svelte/no-navigation-without-resolve -->
				<div class="ml-auto flex items-center gap-2">
					{#if downloadLink}
						<a
							href={downloadLink.url}
							target="_blank"
							rel="noopener noreferrer"
						class="group/{downloadLink.platform} inline-flex items-center gap-2 rounded-lg p-2 text-sm font-medium text-white transition-opacity hover:opacity-90 sm:px-4 sm:py-2"
						style="background-color: {downloadLink.color}"
					>
						<BrandIcon name={downloadLink.platform} class="h-4 w-4" />
						<span class="hidden sm:inline">Download</span>
					</a>
					{/if}
					<a
						href={resolve('/compare', {})}
					class="inline-flex items-center gap-2 rounded-lg bg-primary p-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 sm:px-4 sm:py-2"
				>
					<GitCompareArrows class="h-4 w-4" />
					<span class="hidden sm:inline">Compare</span>
				</a>
				</div>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</div>
		{#if shader.description}
			<p class="mt-2 text-sm text-muted-foreground line-clamp-2">
				{shader.description}
			</p>
		{/if}

		<!-- Inline stats & details -->
			<div class="mt-4 flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted-foreground">
				{#if versions.length > 0 && selectedVersionId}
					<Select.Root
						type="single"
						value={selectedVersionId}
						onValueChange={(v: string) => {
							if (v) void onVersionChange(v);
						}}
					>
						<Select.Trigger
							size="sm"
							class="h-auto gap-1.5 border-0 bg-transparent px-0 py-0 shadow-none ring-0 focus-visible:ring-0 dark:bg-transparent dark:hover:bg-transparent"
						>
							<Layers class="h-3.5 w-3.5" />
							<span class="font-medium text-card-foreground">{selectedVersion ? formatVersion(selectedVersion.version) : 'Version'}</span>
						</Select.Trigger>
						<Select.Content>
							{#each versions as version, i (version.id)}
								<Select.Item
									value={version.id}
									class={version.captureCount === 0 ? 'opacity-40' : ''}
								>
									<span class="flex items-center gap-2">
										{#if version.id === shader.preferredVersionId}
											<Pin class="h-3 w-3 text-info" />
										{/if}
										{formatVersion(version.version)}
										{#if i === 0}
											<span
												class="rounded bg-info/15 px-1.5 py-0.5 text-[10px] font-semibold leading-none text-info"
											>
												Latest
											</span>
										{/if}
									<span class="ml-auto text-xs text-muted-foreground">
										{version.captureCount}
										{#if version.upstreamPublishedAt}
											· {formatDate(version.upstreamPublishedAt)}
										{/if}
									</span>
									</span>
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				{/if}
			<span class="inline-flex items-center gap-1.5">
				<Camera class="h-3.5 w-3.5" />
				<span class="font-medium text-card-foreground">{allCaptures.length}</span> scenes
			</span>
				{#if shader.upstreamDownloads}
					<span class="inline-flex items-center gap-1.5">
						<Download class="h-3.5 w-3.5" />
						<span class="font-medium text-card-foreground">{formatNumber(shader.upstreamDownloads)}</span> downloads
					</span>
				{/if}
				{#if shader.viewCount >= 10}
					<span class="inline-flex items-center gap-1.5">
						<Eye class="h-3.5 w-3.5" />
						<span class="font-medium text-card-foreground">{formatNumber(shader.viewCount)}</span> views
					</span>
				{/if}
			{#if hasLinks}
				<span class="hidden w-px self-stretch bg-border sm:block" aria-hidden="true"></span>
			{/if}

				<!-- eslint-disable svelte/no-navigation-without-resolve -->
				{#if shader.websiteUrl}
					<a
						href={shader.websiteUrl}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-1.5 transition-colors hover:text-card-foreground"
					>
						<Globe class="h-3.5 w-3.5" />
						Website
					</a>
				{/if}
				{#if shader.sourceUrl}
					<a
						href={shader.sourceUrl}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-1.5 transition-colors hover:text-card-foreground"
					>
						<Code class="h-3.5 w-3.5" />
						Source
					</a>
				{/if}
				<!-- eslint-enable svelte/no-navigation-without-resolve -->
			</div>

			<!-- Feature & dimension tags -->
			{#if allFeatures.length > 0 || shader.metadata?.hasCustomTextures === true || (shader.metadata?.dimensionSupport && shader.metadata.dimensionSupport.length > 0)}
				<div class="mt-3 flex flex-wrap gap-1.5">
					{#each allFeatures as feature (feature)}
						<span
							class="inline-flex items-center rounded-md bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary ring-1 ring-primary/20"
						>
							{formatFeatureName(feature)}
						</span>
					{/each}
					{#if shader.metadata?.hasCustomTextures}
						<span
							class="inline-flex items-center rounded-md bg-muted/50 px-2 py-0.5 text-xs text-muted-foreground ring-1 ring-border/50"
						>
							Custom Textures
						</span>
					{/if}
					{#if shader.metadata?.dimensionSupport}
						{#each shader.metadata.dimensionSupport as dim (dim)}
							<span
								class="inline-flex items-center rounded-md bg-muted/50 px-2 py-0.5 text-xs text-muted-foreground ring-1 ring-border/50 capitalize"
							>
								{dim.replace('minecraft:', '')}
							</span>
						{/each}
					{/if}
				</div>
			{/if}

			<!-- Profile pill group -->
			{#if extractedProfiles.length > 0}
				<div class="mt-4 inline-flex flex-wrap items-center gap-1 rounded-lg bg-muted/50 p-1 shadow-theme-sm">
					<button
						type="button"
						class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors select-none
							{selectedProfileId === null
								? 'bg-foreground text-background shadow-sm'
								: 'text-muted-foreground hover:text-foreground'}"
						onclick={() => void onProfileChange(null)}
					>
						All
					</button>
					{#each extractedProfiles as profile (profile.id)}
						<button
							type="button"
							class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors select-none
								{selectedProfileId === profile.id
									? 'bg-foreground text-background shadow-sm'
									: 'text-muted-foreground hover:text-foreground'}"
							onclick={() => void onProfileChange(profile.id)}
						>
						{profile.displayName}
					</button>
				{/each}
			</div>
		{/if}
	</div>

		<!-- Screenshot Grid -->
		<SectionBoundary title="Scene captures">
			{#if captures.length > 0}
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
					{#each captures as capture, i (capture.id)}
						<button
							type="button"
							aria-label="View {capture.sceneName ?? 'capture'} in lightbox"
							class="shadow-theme-sm group relative cursor-pointer overflow-hidden rounded-lg border border-border transition-all hover:border-primary"
							onclick={() => {
								_selectedCaptureId = capture.id;
								openLightbox(i);
							}}
					onmouseenter={() => {
						const url = imageUrl(capture.imagePath, 'full');
						if (url) {
								const img = new Image();
								img.src = url;
							}
						}}
						>
						<CaptureImage
							src={capture.imagePath}
							thumbhash={capture.thumbhash}
							preset="card"
							alt="{shader.name} — {capture.sceneName ?? 'capture'}"
								class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
								containerClass="aspect-video"
							/>

							<!-- Hover overlay with badges -->
							<div
								class="absolute inset-0 bg-linear-to-t from-black/70 via-transparent to-transparent sm:opacity-0 sm:transition-opacity sm:duration-200 sm:group-hover:opacity-100"
							>
								<div class="absolute right-0 bottom-0 left-0 p-3">
									<CaptureBadges
										sceneName={capture.sceneName}
										profileName={capture.profileDisplayName}
										version={capture.shaderVersion}
										{formatVersion}
									/>
								</div>
							</div>
						</button>
					{/each}
				</div>

				<!-- Infinite scroll sentinel (client-side slicing, no network) -->
				{#if hasMoreCaptures}
					<div
						use:observeSentinel
						class="flex justify-center py-8"
					>
					</div>
				{/if}
			{:else}
				<div
					class="flex aspect-video items-center justify-center rounded-xl bg-muted text-muted-foreground"
				>
					<div class="text-center">
						<ImageOff class="mx-auto mb-4 h-16 w-16" strokeWidth={1.5} />
						<p class="text-sm">No captures available for this version</p>
					</div>
				</div>
			{/if}
		</SectionBoundary>

		<!-- Profile Details (collapsible) -->
		{#if extractedProfiles.length > 0}
			<div class="mt-8">
				<Collapsible.Root>
					<Collapsible.Trigger
						class="flex items-center gap-1.5 text-sm font-medium text-foreground transition-colors hover:text-foreground/80"
					>
						<ChevronDown
							class="h-4 w-4 transition-transform [[data-state=open]_&]:rotate-180"
						/>
						Profile settings
						<span class="text-xs font-normal text-foreground/60"
							>({extractedProfiles.length})</span
						>
					</Collapsible.Trigger>
					<Collapsible.Content>
						<div class="mt-3 grid gap-3 sm:grid-cols-2">
							{#each extractedProfiles as profile (profile.id)}
								<div class="rounded-lg border border-border bg-card p-3">
									<h3
										class="mb-2 text-sm font-semibold text-card-foreground"
									>
									{profile.displayName}
									</h3>
									<p class="text-xs text-muted-foreground">
										No overrides (default settings)
									</p>
								</div>
							{/each}
						</div>
					</Collapsible.Content>
				</Collapsible.Root>
			</div>
		{/if}

		<!-- Similar Shaders -->
		{#await data.similarShaders then similarShaders}
			{#if similarShaders.length > 0}
				<SectionBoundary title="Similar shaders">
					<div class="mt-10">
						<h2 class="mb-4 text-lg font-semibold text-foreground">
							Similar Shaders
						</h2>

					<!-- Mobile: compact rows -->
					<ItemGrid items={similarShaders.slice(0, 6)} key={(s: ShaderCardShader) => s.id} mode="row" class="md:hidden">
						{#snippet row(shader: ShaderCardShader)}
								<CompactRow
								name={shader.name}
								subtitle={shader.authors[0]?.name ? `by ${shader.authors[0].name}` : undefined}
								image={shader.imagePath}
								thumbhash={shader.thumbhash}
								href={resolve('/shaders/[slug]', { slug: shader.slug })}
								>
									{#snippet metadata()}
										{#if shader.categories && shader.categories.length > 0}
											<div class="flex gap-1">
												{#each shader.categories.slice(0, 3) as category (category)}
													<span class="rounded bg-muted px-1.5 py-0.5 text-[11px] text-muted-foreground">
														{category.name}
													</span>
												{/each}
											</div>
										{/if}
									{/snippet}
									{#snippet trailing()}
										<ChevronRight class="h-4 w-4 text-muted-foreground" />
									{/snippet}
								</CompactRow>
							{/snippet}
						</ItemGrid>

					<!-- Desktop: horizontal scroll carousel -->
					<div class="hidden md:block">
						<OverlayScrollbarsComponent
							element="div"
							options={{ overflow: { y: 'hidden' }, scrollbars: { theme: themeStore.isDark ? 'os-theme-light' : 'os-theme-dark', autoHide: 'leave', autoHideDelay: 300 } }}
							class="os-subtle"
							defer
						>
							<div class="flex gap-4 pb-3">
								{#each similarShaders as shader (shader.id)}
									<div class="w-44 shrink-0">
								<MiniCard
									name={shader.name}
									subtitle={shader.authors[0]?.name ? `by ${shader.authors[0].name}` : undefined}
									image={shader.imagePath}
									thumbhash={shader.thumbhash}
									href={resolve('/shaders/[slug]', { slug: shader.slug })}
									/>
									</div>
								{/each}
							</div>
						</OverlayScrollbarsComponent>
					</div>
					</div>
				</SectionBoundary>
			{/if}
		{/await}
	</div>
{/key}

<!-- Lightbox -->
{#if lightboxOpen && lightboxCaptures.length > 0}
	<Lightbox
		captures={lightboxCaptures}
		currentIndex={lightboxIndex}
		onClose={() => (lightboxOpen = false)}
		onNavigate={(index: number) => {
			lightboxIndex = index;
			_selectedCaptureId = lightboxCaptures[index]?.id ?? null;
		}}
	/>
{/if}
