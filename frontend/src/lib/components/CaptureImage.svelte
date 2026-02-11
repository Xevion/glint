<script lang="ts">
import { cn } from '$lib/utils';
import { IMAGE_PRESETS, type ImagePreset, cfImageSrcset, cfImageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';

interface Props {
	src: string | null | undefined;
	alt: string;
	thumbhash?: string | null;
	/** Aspect ratio (width / height) from resolution data. Reserves space to prevent layout shift. */
	aspectRatio?: number | null;
	preset?: ImagePreset;
	sizes?: string;
	priority?: boolean;
	class?: string;
	containerClass?: string;
	loading?: 'lazy' | 'eager';
	[key: string]: unknown;
}

let {
	src,
	alt,
	thumbhash,
	aspectRatio,
	preset = 'card',
	sizes,
	priority = false,
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
let imgEl = $state<HTMLImageElement | null>(null);

// Reset loaded state only when src genuinely changes
let prevSrc: string | null | undefined;
$effect(() => {
	if (src !== prevSrc) {
		prevSrc = src;
		loaded = false;
	}
});

// Handle images that load from browser cache before effects run
$effect(() => {
	if (imgEl && !loaded && imgEl.complete && imgEl.naturalWidth > 0) {
		loaded = true;
	}
});
</script>

<div
	class={cn('relative overflow-hidden', containerClass)}
	style:background-image={placeholderUrl ? `url(${placeholderUrl})` : undefined}
	style:background-size="cover"
	style:background-position="center"
	style:aspect-ratio={aspectRatio ? String(aspectRatio) : undefined}
>
	{#if !placeholderUrl && !fallbackSrc}
		<div class={cn('h-full w-full bg-muted', className)}></div>
	{/if}
	{#if fallbackSrc}
		<img
			bind:this={imgEl}
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
