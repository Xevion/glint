<script lang="ts">
import type { ShaderVersionMetadata, ShaderVersionProfile } from '$lib/bindings';
import { formatNumber } from '$lib/utils/display';
import { Camera, Code, Download, ExternalLink, Eye, Globe, Layers } from '@lucide/svelte';

interface Props {
	name: string;
	versionCount: number;
	captureCount: number;
	upstreamDownloads?: number;
	viewCount: number;
	metadata?: ShaderVersionMetadata;
	profiles: ShaderVersionProfile[];
	websiteUrl?: string;
	modrinthId?: string;
	curseforgeId?: string;
	sourceUrl?: string;
	class?: string;
}

let {
	name: _name,
	versionCount,
	captureCount,
	upstreamDownloads,
	viewCount,
	metadata,
	profiles,
	websiteUrl,
	modrinthId,
	curseforgeId,
	sourceUrl,
	class: className
}: Props = $props();

// Collect feature flags from metadata
const allFeatures = $derived.by(() => {
	if (!metadata) return [];
	const features: string[] = [];
	if (metadata.iris_features_required) features.push(...metadata.iris_features_required);
	if (metadata.iris_features_optional) features.push(...metadata.iris_features_optional);
	return features;
});

const hasLinks = $derived(!!websiteUrl || !!modrinthId || !!curseforgeId || !!sourceUrl);

function formatFeatureName(feature: string): string {
	return feature.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
}
</script>

<aside class="rounded-xl border border-border bg-card p-4 {className ?? ''}">
	<!-- Stats -->
	<div>
		<h3 class="mb-1.5 text-xs font-semibold tracking-wide text-card-foreground uppercase">Stats</h3>
		<dl class="space-y-1 text-sm">
			<div class="flex items-center justify-between">
				<dt class="flex items-center gap-1.5 text-muted-foreground">
					<Layers class="h-3.5 w-3.5" />
					Versions
				</dt>
				<dd class="font-medium text-card-foreground">{versionCount}</dd>
			</div>
			<div class="flex items-center justify-between">
				<dt class="flex items-center gap-1.5 text-muted-foreground">
					<Camera class="h-3.5 w-3.5" />
					Scenes
				</dt>
				<dd class="font-medium text-card-foreground">{captureCount}</dd>
			</div>
			{#if upstreamDownloads}
				<div class="flex items-center justify-between">
					<dt class="flex items-center gap-1.5 text-muted-foreground">
						<Download class="h-3.5 w-3.5" />
						Downloads
					</dt>
					<dd class="font-medium text-card-foreground">{formatNumber(upstreamDownloads)}</dd>
				</div>
			{/if}
			{#if viewCount >= 10}
				<div class="flex items-center justify-between">
					<dt class="flex items-center gap-1.5 text-muted-foreground">
						<Eye class="h-3.5 w-3.5" />
						Views
					</dt>
					<dd class="font-medium text-card-foreground">{formatNumber(viewCount)}</dd>
				</div>
			{/if}
		</dl>
	</div>

	<!-- Features -->
	{#if allFeatures.length > 0 || metadata?.has_custom_textures}
		<div class="mt-3 border-t border-card-foreground/10 pt-3">
			<h3 class="mb-1.5 text-xs font-semibold tracking-wide text-card-foreground uppercase">
				Features
			</h3>
			<div class="flex flex-wrap gap-1.5">
				{#each allFeatures as feature (feature)}
					<span
						class="inline-flex items-center rounded-md bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary ring-1 ring-primary/20"
					>
						{formatFeatureName(feature)}
					</span>
				{/each}
				{#if metadata?.has_custom_textures}
					<span
						class="inline-flex items-center rounded-md bg-muted/50 px-2 py-0.5 text-xs text-muted-foreground ring-1 ring-border/50"
					>
						Custom Textures
					</span>
				{/if}
			</div>
		</div>
	{/if}

	<!-- Dimension support -->
	{#if metadata?.dimension_support && metadata.dimension_support.length > 0}
		<div class="mt-3 border-t border-card-foreground/10 pt-3">
			<h3 class="mb-1.5 text-xs font-semibold tracking-wide text-card-foreground uppercase">
				Dimensions
			</h3>
			<div class="flex flex-wrap gap-1.5">
				{#each metadata.dimension_support as dim (dim)}
					<span
						class="inline-flex items-center rounded-md bg-muted/50 px-2 py-0.5 text-xs text-muted-foreground ring-1 ring-border/50 capitalize"
					>
						{dim.replace('minecraft:', '')}
					</span>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Profiles -->
	{#if profiles.length > 0}
		<div class="mt-3 border-t border-card-foreground/10 pt-3">
			<h3 class="mb-1.5 text-xs font-semibold tracking-wide text-card-foreground uppercase">
				Profiles
			</h3>
			<ul class="space-y-0.5">
				{#each profiles as profile (profile.id)}
					<li class="text-sm text-card-foreground">
						{profile.label ?? profile.name}
						{#if profile.description}
							<span class="text-xs text-muted-foreground"> — {profile.description}</span>
						{/if}
					</li>
				{/each}
			</ul>
		</div>
	{/if}

	<!-- Links -->
	{#if hasLinks}
		<div class="mt-3 border-t border-card-foreground/10 pt-3">
			<h3 class="mb-1.5 text-xs font-semibold tracking-wide text-card-foreground uppercase">
				Links
			</h3>
			<!-- eslint-disable svelte/no-navigation-without-resolve -->
			<dl class="space-y-1 text-sm">
				{#if modrinthId}
					<div class="flex items-center justify-between">
						<dt class="flex items-center gap-1.5 text-muted-foreground">
							<Download class="h-3.5 w-3.5" />
							Modrinth
						</dt>
						<dd>
							<a
								href="https://modrinth.com/shader/{modrinthId}"
								target="_blank"
								rel="noopener noreferrer"
								class="text-muted-foreground transition-colors hover:text-card-foreground"
							>
								<ExternalLink class="h-3.5 w-3.5" />
							</a>
						</dd>
					</div>
				{/if}
				{#if curseforgeId}
					<div class="flex items-center justify-between">
						<dt class="flex items-center gap-1.5 text-muted-foreground">
							<Download class="h-3.5 w-3.5" />
							CurseForge
						</dt>
						<dd>
							<a
								href="https://www.curseforge.com/minecraft/shaders/{curseforgeId}"
								target="_blank"
								rel="noopener noreferrer"
								class="text-muted-foreground transition-colors hover:text-card-foreground"
							>
								<ExternalLink class="h-3.5 w-3.5" />
							</a>
						</dd>
					</div>
				{/if}
				{#if websiteUrl}
					<div class="flex items-center justify-between">
						<dt class="flex items-center gap-1.5 text-muted-foreground">
							<Globe class="h-3.5 w-3.5" />
							Website
						</dt>
						<dd>
							<a
								href={websiteUrl}
								target="_blank"
								rel="noopener noreferrer"
								class="text-muted-foreground transition-colors hover:text-card-foreground"
							>
								<ExternalLink class="h-3.5 w-3.5" />
							</a>
						</dd>
					</div>
				{/if}
				{#if sourceUrl}
					<div class="flex items-center justify-between">
						<dt class="flex items-center gap-1.5 text-muted-foreground">
							<Code class="h-3.5 w-3.5" />
							Source
						</dt>
						<dd>
							<a
								href={sourceUrl}
								target="_blank"
								rel="noopener noreferrer"
								class="text-muted-foreground transition-colors hover:text-card-foreground"
							>
								<ExternalLink class="h-3.5 w-3.5" />
							</a>
						</dd>
					</div>
				{/if}
			</dl>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</div>
	{/if}
</aside>
