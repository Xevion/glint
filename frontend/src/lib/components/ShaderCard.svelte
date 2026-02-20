<script lang="ts" module>
import { graphql, type ResultOf } from '$lib/graphql';

/** Shared fragment for shader card fields — used by browse, home, and similar queries. */
export const ShaderCardFragment = graphql(`
	fragment ShaderCardFields on ShaderNode @_unmask {
		id
		name
		slug
		description
		imagePath
		thumbhash
		latestVersion
		modrinthId
		curseforgeId
		websiteUrl
		authors {
			name
		}
		categories {
			id
			name
		}
		features {
			id
			name
		}
	}
`);

/** Type derived from the ShaderCardFields fragment via gql.tada. */
export type ShaderCardShader = ResultOf<typeof ShaderCardFragment>;
</script>

<script lang="ts">
import { resolve } from '$app/paths';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ImageOverlay } from '$lib/components/ui/image-overlay';
import { cn } from '$lib/utils';
import { formatVersion, getCurseforgeUrl, getModrinthUrl } from '$lib/utils/format';
import { ExternalLink } from '@lucide/svelte';
import BrandIcon from './icons/BrandIcon.svelte';

interface Props {
	shader: ShaderCardShader;
	class?: string;
}

let { shader, class: className }: Props = $props();

const modrinthUrl = $derived(getModrinthUrl(shader.modrinthId));
const curseforgeUrl = $derived(getCurseforgeUrl(shader.curseforgeId));
</script>

<div
	class={cn(
		'group relative flex h-full flex-col overflow-hidden rounded-xl bg-card transition-all duration-300',
		'border border-border shadow-theme-sm',
		'hover:-translate-y-1 hover:border-primary/50 hover:shadow-theme-lg',
		className
	)}
>
	<!-- Thumbnail Image -->
	<div class="relative">
		<CaptureImage
		src={shader.imagePath}
		thumbhash={shader.thumbhash}
			preset="card"
			alt="{shader.name} preview"
			class="h-full w-full object-cover"
			containerClass="aspect-video w-full overflow-hidden"

		/>

		<!-- Gradient overlay -->
		<ImageOverlay />
	</div>

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-2 px-3 pt-2.5 pb-2">
		<!-- Header -->
		<div class="flex items-baseline gap-x-2 overflow-hidden">
			<a
				href={resolve('/shaders/[slug]', { slug: shader.slug })}
				class="block truncate text-base font-semibold text-card-foreground transition-colors hover:text-primary focus-visible:outline-2 focus-visible:outline-primary focus-visible:outline-offset-2 after:absolute after:inset-0 after:content-['']"
			>
				{shader.name}
			</a>
			<span class="shrink-0 text-sm text-muted-foreground">by <span class="font-medium text-card-foreground">{shader.authors[0]?.name ?? 'Unknown'}</span></span>
		</div>

		<!-- Description -->
		{#if shader.description}
			<p class="line-clamp-2 flex-1 text-sm text-muted-foreground">
				{shader.description}
			</p>
		{:else}
			<p class="flex-1 text-sm text-muted-foreground/50 italic">No description available</p>
		{/if}

		<!-- Tags: Style & Features -->
		{#if shader.categories.length > 0 || shader.features.length > 0}
			<div class="flex flex-wrap gap-1.5">
				{#each shader.categories as category (category.id)}
					<span
						class="inline-flex items-center rounded-md bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary capitalize ring-1 ring-primary/20"
					>
						{category.name}
					</span>
				{/each}
				{#each shader.features as feature (feature.id)}
					<span
						class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
					>
						{feature.name}
					</span>
				{/each}
			</div>
		{/if}

		<!-- Footer: Version & Links -->
		<div class="mt-auto flex items-center justify-between border-t border-border pt-2">
			<div class="flex min-w-0 items-center gap-3">
				<!-- Version -->
				<span class="inline-flex shrink-0 items-center gap-1 text-xs">
					<span class="font-medium text-card-foreground">{shader.latestVersion ? formatVersion(shader.latestVersion) : '\u2014'}</span>
				</span>
			</div>

			<!-- External links don't need resolve() - only internal app routes -->
			<!-- eslint-disable svelte/no-navigation-without-resolve -->
			<div class="relative z-10 flex items-center gap-1">
				{#if modrinthUrl}
					<a
						href={modrinthUrl}
						target="_blank"
						rel="noopener noreferrer"
						data-external-link
						class="group/modrinth inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
						title="View on Modrinth"
					>
						<BrandIcon name="modrinth" colorOnHover />
					</a>
				{/if}
				{#if curseforgeUrl}
					<a
						href={curseforgeUrl}
						target="_blank"
						rel="noopener noreferrer"
						data-external-link
						class="group/curseforge inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
						title="View on CurseForge"
					>
						<BrandIcon name="curseforge" colorOnHover />
					</a>
				{/if}
			{#if shader.websiteUrl}
				<a
					href={shader.websiteUrl}
						target="_blank"
						rel="noopener noreferrer"
						data-external-link
						class="inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
						title="Visit website"
					>
						<ExternalLink class="h-4 w-4" />
					</a>
				{/if}
			</div>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</div>
	</div>
</div>
