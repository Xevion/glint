<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { cn } from '$lib/utils';

interface Props {
	/** Display name */
	name: string;
	/** Secondary text (author, subtitle, etc.) */
	subtitle?: string;
	/** Thumbnail image URL */
	image?: string;
	/** Thumbhash for image placeholder */
	thumbhash?: string;
	/** Link destination */
	href?: string;
	/** Custom CSS class */
	class?: string;
}

let { name, subtitle, image, thumbhash, href, class: className }: Props = $props();

const cardClasses =
	'group flex flex-col overflow-hidden rounded-lg border border-border bg-card transition-colors';
</script>

{#snippet cardContent()}
	<CaptureImage
		src={image}
		{thumbhash}
		alt="{name} thumbnail"
		preset="thumbnail"
		sizes="176px"
		class="h-full w-full object-cover"
		containerClass="aspect-video w-full overflow-hidden"
	/>
	<div class="flex min-w-0 flex-col gap-0.5 px-2.5 py-2">
		<span class="truncate text-sm font-medium text-card-foreground">{name}</span>
		{#if subtitle}
			<span class="truncate text-xs text-muted-foreground">{subtitle}</span>
		{/if}
	</div>
{/snippet}

{#if href}
	<a
		{href}
		class={cn(cardClasses, 'hover:border-primary/50', className)}
	>
		{@render cardContent()}
	</a>
{:else}
	<div class={cn(cardClasses, className)}>
		{@render cardContent()}
	</div>
{/if}
