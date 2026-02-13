<script lang="ts">
import { browser } from '$app/environment';
import { resolve } from '$app/paths';
import { formatVersion } from '$lib/utils/display';
import { cfImageSrcset, cfImageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';
import { cubicInOut } from 'svelte/easing';
import { crossfade } from 'svelte/transition';
import type { Orientation, SliderSide } from './types';

interface Props {
	side: SliderSide;
	position: 'left' | 'right';
	clipPath: string;
	orientation: Orientation;
	willChange: boolean;
}

let { side, position, clipPath, orientation, willChange }: Props = $props();

let loaded = $state(false);

const placeholder = $derived(browser ? decodeThumbhash(side.thumbhash) : null);
const src = $derived(cfImageUrl(side.image, 'hero'));
const srcset = $derived(cfImageSrcset(side.image, 'hero'));

// Reset loaded state when the image URL changes
$effect(() => {
	void side.image;
	loaded = false;
});

const href = $derived(side.slug ? resolve('/shaders/[slug]', { slug: side.slug }) : undefined);
const detail = $derived.by(() => {
	if (side.author && side.version) return `by ${side.author}, ${formatVersion(side.version)}`;
	if (side.version) return formatVersion(side.version);
	if (side.author) return `by ${side.author}`;
	return undefined;
});

const horizontalClass = $derived(position === 'left' ? 'left-4' : 'right-4');
const isTop = $derived(orientation === (position === 'left' ? 'horizontal' : 'diagonal'));
const verticalClass = $derived(isTop ? 'top-4' : 'bottom-4');

// Crossfade pairs outgoing/incoming labels so the label slides between positions
const [send, receive] = crossfade({
	duration: 400,
	easing: cubicInOut,
	fallback() {
		return { duration: 0 };
	}
});
</script>

<div
	class="absolute inset-0"
	class:will-change-[clip-path]={willChange}
	style:clip-path={clipPath}
>
	{#if placeholder ?? side.thumbhash}
		<div
			class="absolute inset-0 bg-cover bg-center transition-opacity duration-300"
			class:opacity-0={loaded}
			data-thumbhash={!placeholder ? (side.thumbhash ?? undefined) : undefined}
			style:background-image={placeholder ? `url(${placeholder})` : undefined}
		></div>
	{/if}
	<img
		{src}
		{srcset}
		sizes="(min-width: 1024px) 66vw, 100vw"
		alt={side.label || `${position} comparison`}
		class="pointer-events-none absolute inset-0 h-full w-full object-cover transition-opacity duration-300"
		class:opacity-0={!loaded}
		loading="eager"
		decoding="async"
		fetchpriority="high"
		draggable="false"
		onload={() => (loaded = true)}
	/>
	{#if side.label}
		{#key isTop}
			<a
				in:receive={{ key: position }}
				out:send={{ key: position }}
				{href}
				class="pointer-events-auto absolute z-20 block rounded-md border border-white/15 bg-black/60 px-3 py-1.5 backdrop-blur-sm hover:border-white/30 hover:bg-black/70 {horizontalClass} {verticalClass}"
			>
				<span class="text-sm font-medium text-white">{side.label}</span>
				{#if detail}
					<div class="hidden text-xs text-white/70 md:block">{detail}</div>
				{/if}
			</a>
		{/key}
	{/if}
</div>
