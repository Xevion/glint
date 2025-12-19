<script lang="ts">
	import type { PerformanceTier } from '$lib/data/mock';
	import { Feather, Leaf, Scales, Lightning, Fire } from 'phosphor-svelte';

	interface Props {
		tier: PerformanceTier;
		class?: string;
		size?: number | string;
		weight?: 'thin' | 'light' | 'regular' | 'bold' | 'fill' | 'duotone';
	}

	let { tier, class: className, size = 12, weight = 'fill' }: Props = $props();
</script>

{#if tier === 'potato'}
	<Feather class={className} {size} {weight} />
{:else if tier === 'low'}
	<Leaf class={className} size={Number(size) * 1.1} weight="bold" />
{:else if tier === 'medium'}
	<Scales class={className} size={Number(size) * 1.15} {weight} />
{:else if tier === 'high'}
	<Lightning class={className} size={Number(size) * 1.1} weight="bold" />
{:else if tier === 'ultra'}
	<Fire class={className} {size} weight="bold" />
{/if}
