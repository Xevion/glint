<script lang="ts">
import { resolve } from '$app/paths';
import { createApiClient } from '$lib/api';
import type { ShaderListItem } from '$lib/bindings';

import CaptureBadges from '$lib/components/CaptureBadges.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { CompactRow, ItemGrid, MiniCard } from '$lib/components/item-grid';
import Lightbox from '$lib/components/Lightbox.svelte';
import Meta from '$lib/components/Meta.svelte';
import SectionBoundary from '$lib/components/SectionBoundary.svelte';
import BrandIcon from '$lib/components/icons/BrandIcon.svelte';
import * as Collapsible from '$lib/components/ui/collapsible';
import * as Select from '$lib/components/ui/select';
import { formatNumber, formatVersion, getCurseforgeUrl, getModrinthUrl } from '$lib/utils/display';
import { imageUrl } from '$lib/utils/image';
import { withRetry } from '$lib/utils/retry';
import { themeStore } from '$lib/stores/theme.svelte';
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
	LoaderCircle
} from '@lucide/svelte';
import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';
import type { GetShaderParams } from '$lib/api/endpoints/shaders';
import { type ShaderDetail, type ShaderDetailCapture, _trimCapture, _trimShader } from './+page.ts';

interface Props {
	data: PageData;
}

let { data }: Props = $props();

// Local override: set when version changes, cleared on navigation
let shaderOverride = $state<ShaderDetail | null>(null);

// Pagination state for captures
const api = createApiClient(fetch);
let allCaptures = $state<ShaderDetailCapture[]>([]);
let capturesPage = $state(1);
let capturesTotal = $state(0);
let capturesLoading = $state(false);
const capturesPageSize = 24;
const hasMoreCaptures = $derived(allCaptures.length < capturesTotal);

// Initialize/reset captures from SSR data when shader page data changes
$effect(() => {
	const capturesData = data.capturesData;
	allCaptures = capturesData.items;
	capturesPage = capturesData.page;
	capturesTotal = capturesData.total;
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
let captures = $derived(allCaptures);

// Version selection
const versions = $derived(shader.versions);
const defaultVersionId = $derived(
	versions.find((v) => v.capture_count > 0)?.id ?? versions[0]?.id ?? null
);
let versionOverride = $state<string | null>(null);
const selectedVersionId = $derived(versionOverride ?? defaultVersionId);
const selectedVersion = $derived(
	versions.find((v) => v.id === selectedVersionId) ?? versions[0] ?? null
);

// Profiles from extraction data for the selected version
const extractedProfiles = $derived(shader.profiles ?? []);

// Header metadata derivations
const allFeatures = $derived.by(() => {
	const m = shader.metadata;
	if (!m) return [];
	const features: string[] = [];
	if (m.iris_features_required) features.push(...m.iris_features_required);
	if (m.iris_features_optional) features.push(...m.iris_features_optional);
	return features;
});
const modrinthUrl = $derived(getModrinthUrl(shader.modrinth_id));
const curseforgeUrl = $derived(getCurseforgeUrl(shader.curseforge_id));
const downloadLink = $derived.by(() => {
	if (modrinthUrl) return { url: modrinthUrl, platform: 'modrinth' as const, color: '#00af5c' };
	if (curseforgeUrl)
		return { url: curseforgeUrl, platform: 'curseforge' as const, color: '#f16436' };
	return null;
});
const hasLinks = $derived(!!shader.website_url || !!shader.source_url);

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

// Guards stale responses from racing version fetches
let fetchGeneration = 0;

async function onVersionChange(versionId: string) {
	versionOverride = versionId;
	_selectedCaptureId = null;
	selectedProfileId = null;
	const generation = ++fetchGeneration;

	const [result, capturesResult] = await Promise.all([
		withRetry(() => api.shaders.getShader(shader.slug, { versionId })),
		api.shaders.listCaptures(shader.slug, {
			page: 1,
			pageSize: capturesPageSize,
			versionId
		})
	]);
	if (generation !== fetchGeneration) return;

	result.match({
		Ok: (updated) => {
			shaderOverride = _trimShader(updated);
		},
		Err: (err) => {
			console.warn('Failed to fetch shader version:', err.message);
		}
	});

	capturesResult.match({
		Ok: (p) => {
			allCaptures = p.items.map(_trimCapture);
			capturesPage = p.page;
			capturesTotal = p.total;
		},
		Err: (err) => {
			console.warn('Failed to fetch captures for version:', err.message);
		}
	});
}

async function onProfileChange(profileId: string | null) {
	selectedProfileId = profileId;
	_selectedCaptureId = null;
	const generation = ++fetchGeneration;
	const shaderParams: GetShaderParams = {};
	if (selectedVersionId) shaderParams.versionId = selectedVersionId;
	if (profileId) shaderParams.profileId = profileId;

	const [result, capturesResult] = await Promise.all([
		withRetry(() => api.shaders.getShader(shader.slug, shaderParams)),
		api.shaders.listCaptures(shader.slug, {
			page: 1,
			pageSize: capturesPageSize,
			versionId: selectedVersionId ?? undefined,
			profileId: profileId ?? undefined
		})
	]);
	if (generation !== fetchGeneration) return;

	result.match({
		Ok: (updated) => {
			shaderOverride = _trimShader(updated);
		},
		Err: (err) => {
			console.warn('Failed to fetch shader captures for profile:', err.message);
		}
	});

	capturesResult.match({
		Ok: (p) => {
			allCaptures = p.items.map(_trimCapture);
			capturesPage = p.page;
			capturesTotal = p.total;
		},
		Err: (err) => {
			console.warn('Failed to fetch captures for profile:', err.message);
		}
	});
}

async function loadMoreCaptures() {
	if (capturesLoading || !hasMoreCaptures) return;
	capturesLoading = true;
	const generation = ++fetchGeneration;
	const nextPage = capturesPage + 1;
	const result = await api.shaders.listCaptures(shader.slug, {
		page: nextPage,
		pageSize: capturesPageSize,
		versionId: selectedVersionId ?? undefined,
		profileId: selectedProfileId ?? undefined
	});
	if (generation !== fetchGeneration) {
		capturesLoading = false;
		return;
	}
	result.match({
		Ok: (paginated) => {
			allCaptures = [...allCaptures, ...paginated.items.map(_trimCapture)];
			capturesPage = paginated.page;
			capturesTotal = paginated.total;
		},
		Err: (err) => {
			console.warn('Failed to load more captures:', err.message);
		}
	});
	capturesLoading = false;
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
const ogImage = $derived(firstCapture?.image_path ?? null);
const ogDescription = $derived.by(() => {
	const parts = [`${shader.name} shader for Minecraft`];
	if (selectedVersion) parts.push(`v${selectedVersion.version}`);
	if (capturesTotal > 0) parts.push(`${capturesTotal} screenshot${capturesTotal === 1 ? '' : 's'}`);
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
				{#if shader.icon_url && !iconErrored}
					<img
						src={shader.icon_url}
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
									class={version.capture_count === 0 ? 'opacity-40' : ''}
								>
									<span class="flex items-center gap-2">
										{formatVersion(version.version)}
										{#if i === 0 && version.capture_count > 0}
											<span
												class="rounded bg-info/15 px-1.5 py-0.5 text-[10px] font-semibold leading-none text-info"
											>
												Latest
											</span>
										{/if}
									</span>
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				{/if}
			<span class="inline-flex items-center gap-1.5">
				<Camera class="h-3.5 w-3.5" />
				<span class="font-medium text-card-foreground">{capturesTotal}</span> scenes
			</span>
				{#if shader.upstream_downloads}
					<span class="inline-flex items-center gap-1.5">
						<Download class="h-3.5 w-3.5" />
						<span class="font-medium text-card-foreground">{formatNumber(shader.upstream_downloads)}</span> downloads
					</span>
				{/if}
				{#if shader.view_count >= 10}
					<span class="inline-flex items-center gap-1.5">
						<Eye class="h-3.5 w-3.5" />
						<span class="font-medium text-card-foreground">{formatNumber(shader.view_count)}</span> views
					</span>
				{/if}
			{#if hasLinks}
				<span class="hidden w-px self-stretch bg-border sm:block" aria-hidden="true"></span>
			{/if}

				<!-- eslint-disable svelte/no-navigation-without-resolve -->
				{#if shader.website_url}
					<a
						href={shader.website_url}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-1.5 transition-colors hover:text-card-foreground"
					>
						<Globe class="h-3.5 w-3.5" />
						Website
					</a>
				{/if}
				{#if shader.source_url}
					<a
						href={shader.source_url}
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
			{#if allFeatures.length > 0 || shader.metadata?.has_custom_textures === true || (shader.metadata?.dimension_support && shader.metadata.dimension_support.length > 0)}
				<div class="mt-3 flex flex-wrap gap-1.5">
					{#each allFeatures as feature (feature)}
						<span
							class="inline-flex items-center rounded-md bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary ring-1 ring-primary/20"
						>
							{formatFeatureName(feature)}
						</span>
					{/each}
					{#if shader.metadata?.has_custom_textures}
						<span
							class="inline-flex items-center rounded-md bg-muted/50 px-2 py-0.5 text-xs text-muted-foreground ring-1 ring-border/50"
						>
							Custom Textures
						</span>
					{/if}
					{#if shader.metadata?.dimension_support}
						{#each shader.metadata.dimension_support as dim (dim)}
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
						{profile.display_name}
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
							aria-label="View {capture.scene_name ?? 'capture'} in lightbox"
							class="shadow-theme-sm group relative cursor-pointer overflow-hidden rounded-lg border border-border transition-all hover:border-primary"
							onclick={() => {
								_selectedCaptureId = capture.id;
								openLightbox(i);
							}}
					onmouseenter={() => {
						const url = imageUrl(capture.image_path, 'full');
						if (url) {
								const img = new Image();
								img.src = url;
							}
						}}
						>
						<CaptureImage
							src={capture.image_path}
							thumbhash={capture.thumbhash}
							preset="card"
							alt="{shader.name} — {capture.scene_name ?? 'capture'}"
								class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
								containerClass="aspect-video"
							/>

							<!-- Hover overlay with badges -->
							<div
								class="absolute inset-0 bg-linear-to-t from-black/70 via-transparent to-transparent sm:opacity-0 sm:transition-opacity sm:duration-200 sm:group-hover:opacity-100"
							>
								<div class="absolute right-0 bottom-0 left-0 p-3">
									<CaptureBadges
										sceneName={capture.scene_name}
										profileName={capture.profile_name}
										version={capture.shader_version}
										{formatVersion}
									/>
								</div>
							</div>
						</button>
					{/each}
				</div>

				<!-- Infinite scroll sentinel -->
				{#if hasMoreCaptures || capturesLoading}
					<div
						use:observeSentinel
						class="flex justify-center py-8"
					>
						{#if capturesLoading}
							<LoaderCircle class="h-6 w-6 animate-spin text-muted-foreground" />
						{/if}
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
									{profile.display_name}
									</h3>
									{#if Object.keys(profile.options).length > 0}
										<dl class="space-y-0.5 text-xs">
											{#each Object.entries(profile.options) as [key, value] (key)}
												<div class="flex justify-between gap-2">
													<dt
														class="truncate font-mono text-muted-foreground"
													>
														{key}
													</dt>
													<dd
														class="shrink-0 font-mono font-medium text-card-foreground"
													>
														{value}
													</dd>
												</div>
											{/each}
										</dl>
									{:else}
										<p class="text-xs text-muted-foreground">
											No overrides (default settings)
										</p>
									{/if}
								</div>
							{/each}
						</div>
					</Collapsible.Content>
				</Collapsible.Root>
			</div>
		{/if}

		<!-- Similar Shaders -->
		{#if data.similarShaders.length > 0}
			<SectionBoundary title="Similar shaders">
				<div class="mt-10">
					<h2 class="mb-4 text-lg font-semibold text-foreground">
						Similar Shaders
					</h2>

				<!-- Mobile: compact rows -->
				<ItemGrid items={data.similarShaders.slice(0, 6)} key={(s: ShaderListItem) => s.id} mode="row" class="md:hidden">
					{#snippet row(shader: ShaderListItem)}
							<CompactRow
							name={shader.name}
							subtitle={shader.authors[0]?.name ? `by ${shader.authors[0].name}` : undefined}
							image={shader.image_path}
							thumbhash={shader.thumbhash}
							href={resolve('/shaders/[slug]', { slug: shader.slug })}
							>
								{#snippet metadata()}
									{#if shader.categories && shader.categories.length > 0}
										<div class="flex gap-1">
											{#each shader.categories.slice(0, 3) as category (category)}
												<span class="rounded bg-muted px-1.5 py-0.5 text-[11px] text-muted-foreground">
													{category}
												</span>
											{/each}
										</div>
									{:else if shader.upstream_downloads}
										<span class="text-xs text-muted-foreground">
											{formatNumber(shader.upstream_downloads)} downloads
										</span>
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
							{#each data.similarShaders as shader (shader.id)}
								<div class="w-44 shrink-0">
							<MiniCard
								name={shader.name}
								subtitle={shader.authors[0]?.name ? `by ${shader.authors[0].name}` : undefined}
								image={shader.image_path}
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
	</div>
{/key}

<!-- Lightbox -->
{#if lightboxOpen && captures.length > 0}
	<Lightbox
		{captures}
		currentIndex={lightboxIndex}
		onClose={() => (lightboxOpen = false)}
		onNavigate={(index: number) => {
			lightboxIndex = index;
			_selectedCaptureId = captures[index]?.id ?? null;
		}}
	/>
{/if}
