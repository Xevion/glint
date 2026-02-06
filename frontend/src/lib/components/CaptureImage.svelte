<script lang="ts">
import { type ImagePreset, cfImageUrl } from '$lib/utils/image';

interface Props {
	src: string | null | undefined;
	alt: string;
	preset?: ImagePreset;
	class?: string;
	loading?: 'lazy' | 'eager';
	[key: string]: unknown;
}

let { src, alt, preset = 'card', class: className, loading = 'lazy', ...rest }: Props = $props();

const transformedSrc = $derived(cfImageUrl(src, preset));
</script>

{#if transformedSrc}
	<img src={transformedSrc} {alt} class={className} {loading} {...rest} />
{/if}
