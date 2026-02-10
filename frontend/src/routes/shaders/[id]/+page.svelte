<script lang="ts">
import { resolve } from '$app/paths';
import type { ShaderWithCaptures } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import Lightbox from '$lib/components/Lightbox.svelte';
import * as Select from '$lib/components/ui/select';
import { formatNumber, formatVersion } from '$lib/utils/display';
import { preloadImage } from '$lib/utils/image';
import { ChevronRight, ImageOff, ExternalLink, Download } from '@lucide/svelte';
import { createApiClient } from '$lib/api';
import { fly } from 'svelte/transition';

interface Props {
	data: { shader: ShaderWithCaptures };
}

let { data }: Props = $props();

// Local override: set when version changes, cleared on navigation
let shaderOverride = $state<ShaderWithCaptures | null>(null);

// Reset user overrides when navigating between shader pages
$effect(() => {
	void data.shader;
	shaderOverride = null;
	versionOverride = null;
	selectedCaptureId = null;
});

// Core data: prefer override (from version change), fall back to page data
const shader = $derived(shaderOverride ?? data.shader);
let captures = $derived(shader.captures);

// Version selection: default is latest version with captures, user override takes precedence
const versions = $derived(shader.versions);
const defaultVersionId = $derived(
	versions.find((v) => v.capture_count > 0)?.id ?? versions[0]?.id ?? null
);
let versionOverride = $state<string | null>(null);
const selectedVersionId = $derived(versionOverride ?? defaultVersionId);

const selectedVersion = $derived(
	versions.find((v) => v.id === selectedVersionId) ?? versions[0] ?? null
);

// Parse profiles from selected version
const profiles = $derived.by(() => {
	const raw = selectedVersion?.supported_profiles;
	if (!raw) return [];
	try {
		const parsed: unknown = JSON.parse(raw);
		return Array.isArray(parsed) ? (parsed as string[]) : [];
	} catch {
		return [];
	}
});

// Hero: derived from user selection, falling back to first capture
let selectedCaptureId = $state<string | null>(null);
const heroCapture = $derived(
	captures.find((c) => c.id === selectedCaptureId) ?? captures[0] ?? null
);

// Lightbox state
let lightboxOpen = $state(false);
let lightboxIndex = $state(0);

// Guards stale responses from racing version fetches
let fetchGeneration = 0;

// Version change handler — re-fetch captures
async function onVersionChange(versionId: string) {
	versionOverride = versionId;
	selectedCaptureId = null;
	const generation = ++fetchGeneration;
	const api = createApiClient(fetch);
	const result = await api.shaders.getShader(shader.slug, { versionId });
	if (generation !== fetchGeneration) return;
	result.match({
		Ok: (updated) => {
			shaderOverride = updated;
		},
		Err: (err) => {
			console.warn('Failed to fetch shader version:', err.message);
		}
	});
}

// Lightbox helpers
function openLightbox(captureIndex: number) {
	lightboxIndex = captureIndex;
	lightboxOpen = true;
}
</script>

{#snippet captureBadges(capture: typeof captures[number], size: 'sm' | 'xs')}
	<div class="flex items-center gap-2">
		{#if capture.scene_name}
			<span
				class="rounded bg-white/20 px-2 {size === 'sm' ? 'py-1 text-sm' : 'py-0.5 text-xs'} font-medium text-white"
			>
				{capture.scene_name}
			</span>
		{/if}
		{#if capture.profile}
			<span
				class="rounded bg-primary px-2 {size === 'sm' ? 'py-1 text-sm' : 'py-0.5 text-xs'} font-medium text-white"
			>
				{capture.profile}
			</span>
		{/if}
		{#if capture.shader_version}
			<span
				class="rounded bg-white/20 px-2 {size === 'sm' ? 'py-1 text-sm' : 'py-0.5 text-xs'} font-medium text-white"
			>
				{formatVersion(capture.shader_version)}
			</span>
		{/if}
	</div>
{/snippet}

{#key shader.id}
		<div class="container mx-auto px-4 py-8">
			<!-- Breadcrumb -->
			<nav
				aria-label="Breadcrumb"
				in:fly|local={{ y: -10, duration: 400 }}
				class="mb-6 flex items-center gap-2 text-sm text-muted-foreground"
			>
				<a href={resolve('/shaders', {})} class="transition-colors hover:text-foreground">
					Shaders
				</a>
				<ChevronRight class="h-4 w-4" strokeWidth={2} />
				<span class="font-medium text-foreground">{shader.name}</span>
			</nav>

			<!-- Header -->
			<div in:fly|local={{ y: 10, duration: 400, delay: 100 }} class="mb-8">
				<div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
					<!-- Left: Icon + Name + Description -->
					<div class="flex items-start gap-4">
						{#if shader.icon_url}
							<img
								src={shader.icon_url}
								alt="{shader.name} icon"
								class="h-14 w-14 rounded-lg object-cover"
							/>
						{/if}
						<div>
							<h1 class="text-3xl font-bold text-foreground">{shader.name}</h1>
							{#if shader.description}
								<p class="mt-1 max-w-2xl text-sm text-foreground/70 dark:text-muted-foreground">
									{shader.description}
								</p>
							{/if}
						</div>
					</div>

					<!-- Right: Version Selector -->
					{#if versions.length > 0 && selectedVersionId}
						<Select.Root
							type="single"
							value={selectedVersionId}
						onValueChange={(v: string) => {
							if (v) void onVersionChange(v);
						}}
						>
							<Select.Trigger class="w-48">
								{selectedVersion ? formatVersion(selectedVersion.version) : 'Select version'}
							</Select.Trigger>
						<Select.Content>
							{#each versions as version, i (version.id)}
								<Select.Item
									value={version.id}
									class={version.capture_count === 0
										? 'opacity-40'
										: ''}
								>
									<span class="flex items-center gap-2">
										{formatVersion(version.version)}
										{#if i === 0 && version.capture_count > 0}
											<span
												class="rounded bg-blue-500/15 px-1.5 py-0.5 text-[10px] font-semibold leading-none text-blue-600 dark:bg-blue-400/15 dark:text-blue-400"
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
				</div>

				<!-- Stats Pills + Profiles + Links -->
				<div class="mt-4 flex flex-wrap items-center gap-3">
					<!-- Stats pills -->
					<span class="rounded-full bg-muted px-3 py-1 text-xs font-medium text-muted-foreground">
						{versions.length} {versions.length === 1 ? 'version' : 'versions'}
					</span>
					<span class="rounded-full bg-muted px-3 py-1 text-xs font-medium text-muted-foreground">
						{captures.length} {captures.length === 1 ? 'scene' : 'scenes'}
					</span>
					{#if shader.upstream_downloads}
						<span
							class="rounded-full bg-muted px-3 py-1 text-xs font-medium text-muted-foreground"
						>
							{formatNumber(shader.upstream_downloads)} downloads
						</span>
					{/if}

					<!-- Profiles -->
					{#each profiles as profile (profile)}
						<span class="rounded-full bg-primary/10 px-3 py-1 text-xs font-medium text-primary">
							{profile}
						</span>
					{/each}

					<!-- Separator -->
					{#if shader.website_url ?? shader.modrinth_id ?? shader.source_url}
						<span class="text-border">|</span>
					{/if}

					<!-- Links -->
					{#if shader.website_url}
						<a
							href={shader.website_url}
							target="_blank"
							rel="noopener noreferrer"
							class="inline-flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
						>
							<ExternalLink class="h-3 w-3" />
							Website
						</a>
					{/if}
					{#if shader.modrinth_id}
						<a
							href="https://modrinth.com/shader/{shader.modrinth_id}"
							target="_blank"
							rel="noopener noreferrer"
							class="inline-flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
						>
							<Download class="h-3 w-3" />
							Modrinth
						</a>
					{/if}
					{#if shader.source_url}
						<a
							href={shader.source_url}
							target="_blank"
							rel="noopener noreferrer"
							class="inline-flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
						>
							<ExternalLink class="h-3 w-3" />
							Source
						</a>
					{/if}
				</div>
			</div>

			<!-- Hero Image -->
			{#if heroCapture}
				<div class="mb-8">
					<button
						type="button"
						class="shadow-theme-lg group relative w-full cursor-pointer overflow-hidden rounded-xl"
					onclick={() => {
						const idx = captures.indexOf(heroCapture);
						if (idx >= 0) openLightbox(idx);
					}}
					onmouseenter={() => preloadImage(heroCapture?.image_url)}
					>
						<!-- Preloads full-res image on hover for instant lightbox display -->
						<CaptureImage
							src={heroCapture.image_url}
							thumbhash={heroCapture.thumbhash}
							preset="hero"
							priority
							alt="{shader.name} — {heroCapture.scene_name ?? 'capture'}"
							class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
							containerClass="aspect-video"
						/>

						<!-- Hover overlay -->
						<div
							class="absolute inset-0 bg-linear-to-t from-black/70 via-transparent to-transparent opacity-0 transition-opacity duration-200 group-hover:opacity-100"
						>
							<div class="absolute right-0 bottom-0 left-0 p-4">
								{@render captureBadges(heroCapture, 'sm')}
							</div>
						</div>
					</button>
				</div>
			{:else}
				<div
					class="mb-8 flex aspect-video items-center justify-center rounded-xl bg-muted text-muted-foreground"
				>
					<div class="text-center">
						<ImageOff class="mx-auto mb-4 h-16 w-16" strokeWidth={1.5} />
						<p class="text-sm">No captures available yet</p>
					</div>
				</div>
			{/if}

			<!-- Scene Grid -->
		{#if captures.length > 1}
			<div>
				<h2 class="mb-4 text-lg font-semibold text-foreground">Scenes</h2>
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
					{#each captures as capture, i (capture.id)}
							<button
								type="button"
								aria-label="View {capture.scene_name ?? 'capture'} in lightbox"
								class="shadow-theme group relative cursor-pointer overflow-hidden rounded-lg border border-border transition-all hover:border-primary"
							onclick={() => {
								selectedCaptureId = capture.id;
								openLightbox(i);
							}}
								onmouseenter={() => preloadImage(capture.image_url)}
							>
								<CaptureImage
									src={capture.image_url}
									thumbhash={capture.thumbhash}
									preset="card"
									alt="{shader.name} — {capture.scene_name ?? 'capture'}"
									class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
									containerClass="aspect-video"
								/>

								<!-- Hover overlay -->
								<div
									class="absolute inset-0 bg-linear-to-t from-black/70 via-transparent to-transparent opacity-0 transition-opacity duration-200 group-hover:opacity-100"
								>
									<div class="absolute right-0 bottom-0 left-0 p-3">
										{@render captureBadges(capture, 'xs')}
									</div>
								</div>
							</button>
						{/each}
					</div>
				</div>
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
			selectedCaptureId = captures[index]?.id ?? null;
		}}
	/>
{/if}
