<script lang="ts">
import { cn } from '$lib/utils';
import { type ImagePreset, cfImageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';

interface Props {
	src: string | null | undefined;
	alt: string;
	thumbhash?: string | null;
	preset?: ImagePreset;
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
	class: className,
	containerClass,
	loading = 'lazy',
	...rest
}: Props = $props();

const transformedSrc = $derived(cfImageUrl(src, preset));
const placeholderUrl = $derived(decodeThumbhash(thumbhash));
let loaded = $state(false);
</script>

<div
	class={cn('relative overflow-hidden', containerClass)}
	style:background-image={placeholderUrl ? `url(${placeholderUrl})` : undefined}
	style:background-size="cover"
	style:background-position="center"
>
	{#if !placeholderUrl && !transformedSrc}
		<div class={cn('h-full w-full bg-muted', className)}></div>
	{/if}
	{#if transformedSrc}
		<img
			src={transformedSrc}
			{alt}
			class={cn(className, 'transition-opacity duration-300', loaded ? 'opacity-100' : 'opacity-0')}
			{loading}
			onload={() => (loaded = true)}
			{...rest}
		/>
	{/if}
</div>
