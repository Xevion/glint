<script lang="ts">
import { cn } from '$lib/utils';
import { cfImageSrcset, cfImageUrl, IMAGE_PRESETS, type ImagePreset } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';

interface Props {
	src: string | null | undefined;
	alt: string;
	thumbhash?: string | null;
	preset?: ImagePreset;
	sizes?: string;
	priority?: boolean;
	lightbox?: boolean;
	class?: string;
	containerClass?: string;
	loading?: 'lazy' | 'eager';
	[key: string]: unknown;
}

let {
	src,
	alt,
	thumbhash,
	preset = 'card',
	sizes,
	priority = false,
	lightbox = false,
	class: className,
	containerClass,
	loading,
	...rest
}: Props = $props();

const srcset = $derived(cfImageSrcset(src, preset));
const fallbackSrc = $derived(cfImageUrl(src, { width: 640, format: 'auto' }));
const resolvedSizes = $derived(sizes ?? IMAGE_PRESETS[preset].sizes);
const placeholderUrl = $derived(decodeThumbhash(thumbhash));
const resolvedLoading = $derived(loading ?? (priority ? 'eager' : 'lazy'));
const resolvedDecoding = $derived(priority ? 'sync' : 'async');

let loaded = $state(false);
let lightboxOpen = $state(false);

// Reset loaded state when src changes
$effect(() => {
	if (src) {
		loaded = false;
	}
});

function handleLightboxKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape') {
		lightboxOpen = false;
	}
}
</script>

<svelte:window onkeydown={lightbox && lightboxOpen ? handleLightboxKeydown : undefined} />

{#if lightbox && lightboxOpen}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex cursor-pointer items-center justify-center bg-black/90 backdrop-blur-sm"
		onclick={() => (lightboxOpen = false)}
		onkeydown={handleLightboxKeydown}
	>
		<img
			src={cfImageUrl(src, 'full')}
			{alt}
			class="max-h-[90vh] max-w-[90vw] object-contain"
			loading="eager"
			decoding="async"
		/>
	</div>
{/if}

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
	class={cn('relative overflow-hidden', containerClass)}
	style:background-image={placeholderUrl ? `url(${placeholderUrl})` : undefined}
	style:background-size="cover"
	style:background-position="center"
	role={lightbox ? 'button' : undefined}
	tabindex={lightbox ? 0 : undefined}
	onclick={lightbox ? () => (lightboxOpen = true) : undefined}
	onkeydown={lightbox
		? (e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					lightboxOpen = true;
				}
			}
		: undefined}
	style:cursor={lightbox ? 'pointer' : undefined}
>
	{#if !placeholderUrl && !fallbackSrc}
		<div class={cn('h-full w-full bg-muted', className)}></div>
	{/if}
	{#if fallbackSrc}
		<img
			src={fallbackSrc}
			srcset={srcset}
			sizes={resolvedSizes}
			{alt}
			class={cn(className, 'transition-opacity duration-300', loaded ? 'opacity-100' : 'opacity-0')}
			loading={resolvedLoading}
			decoding={resolvedDecoding}
			fetchpriority={priority ? 'high' : undefined}
			onload={() => (loaded = true)}
			{...rest}
		/>
	{/if}
</div>
