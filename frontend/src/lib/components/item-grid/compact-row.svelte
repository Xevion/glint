<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { cn } from '$lib/utils';
import type { Snippet } from 'svelte';

interface Props {
	/** Display name */
	name: string;
	/** Secondary text (author, subtitle, etc.) */
	subtitle?: string;
	/** Thumbnail image URL */
	image?: string;
	/** Thumbhash for image placeholder */
	thumbhash?: string;
	/** Called when the row is clicked */
	onclick?: () => void;
	/** Additional metadata rendered inline after the subtitle */
	metadata?: Snippet;
	/** Trailing content (badges, actions, arrow) */
	trailing?: Snippet;
	/** Custom CSS class */
	class?: string;
}

let {
	name,
	subtitle,
	image,
	thumbhash,
	onclick,
	metadata,
	trailing,
	class: className
}: Props = $props();

const baseClasses = 'flex w-full items-center gap-3 px-4 py-2.5 text-left transition-colors';
const interactiveClasses =
	'cursor-pointer hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:outline-none';
</script>

{#snippet content()}
	<!-- Thumbnail -->
	{#if image}
		<CaptureImage
			src={image}
			{thumbhash}
			alt="{name} thumbnail"
			class="h-full w-full object-cover"
			containerClass="h-10 w-10 shrink-0 overflow-hidden rounded-md"
		/>
	{:else}
		<div class="h-10 w-10 shrink-0 rounded-md bg-muted"></div>
	{/if}

	<!-- Name + subtitle -->
	<div class="min-w-0 flex-1">
		<div class="flex items-baseline gap-2">
			<span class="truncate text-sm font-medium text-card-foreground">{name}</span>
			{#if subtitle}
				<span class="shrink-0 text-xs text-muted-foreground">{subtitle}</span>
			{/if}
		</div>
		{#if metadata}
			<div class="mt-0.5 flex items-center gap-2 text-xs text-muted-foreground">
				{@render metadata()}
			</div>
		{/if}
	</div>

	<!-- Trailing content -->
	{#if trailing}
		<div class="shrink-0">
			{@render trailing()}
		</div>
	{/if}
{/snippet}

{#if onclick}
	<button type="button" {onclick} class={cn(baseClasses, interactiveClasses, className)}>
		{@render content()}
	</button>
{:else}
	<div class={cn(baseClasses, className)}>
		{@render content()}
	</div>
{/if}
