<script lang="ts">
import { browser } from '$app/environment';
import { cn } from '$lib/utils';
import { IMAGE_PRESETS, type ImagePreset, imageSrcset, imageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';
import { ImageOff } from '@lucide/svelte';

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

const srcset = $derived(imageSrcset(src, preset));
const fallbackSrc = $derived(imageUrl(src, { width: 640, format: 'auto' }));
const resolvedSizes = $derived(sizes ?? IMAGE_PRESETS[preset].sizes);
const placeholderUrl = $derived(browser ? decodeThumbhash(thumbhash) : null);
const resolvedLoading = $derived(loading ?? (priority ? 'eager' : 'lazy'));
const resolvedDecoding = $derived(priority ? 'sync' : 'async');

let loaded = $state(false);
let errored = $state(false);
let imgEl = $state<HTMLImageElement | null>(null);

// Reset loaded/errored state when src changes
let prevSrc: string | null | undefined;
$effect(() => {
	if (src !== prevSrc) {
		prevSrc = src;
		loaded = false;
		errored = false;
	}
});

// Handle images that load from browser cache before effects run
$effect(() => {
	if (imgEl && !loaded && !errored && imgEl.complete && imgEl.naturalWidth > 0) {
		loaded = true;
	}
});
</script>

<div
	class={cn('relative overflow-hidden', containerClass)}
	data-thumbhash={thumbhash ?? undefined}
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
			class={cn(className, 'transition-opacity duration-300', loaded && !errored ? 'opacity-100' : 'opacity-0')}
			loading={resolvedLoading}
			decoding={resolvedDecoding}
			fetchpriority={priority ? 'high' : undefined}
			onload={() => (loaded = true)}
			onerror={() => (errored = true)}
			{...rest}
		/>
	{/if}
	{#if errored}
		<div class="absolute inset-0 flex flex-col items-center justify-center bg-black/30 text-white/70">
			<ImageOff class="h-6 w-6" strokeWidth={1.5} />
			<span class="mt-1 text-xs">Image unavailable</span>
		</div>
	{/if}
</div>
