<script lang="ts">
import type { Column } from '@tanstack/table-core';
import { ArrowDown, ArrowUp, ChevronsUpDown } from '@lucide/svelte';
import { cn } from '$lib/utils';
import { Button } from '$lib/components/ui/button';

interface Props {
	// Column<TData> is contravariant on TData, so we accept any to avoid
	// type incompatibilities when passed via renderComponent from column defs.
	// We only call sorting methods which don't depend on TData.
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	column: Column<any, any>;
	title: string;
	class?: string;
}

let { column, title, class: className }: Props = $props();
</script>

{#if !column.getCanSort()}
	<div class={cn(className)}>{title}</div>
{:else}
	<Button
		variant="ghost"
		size="sm"
		class={cn('-ml-3 h-8 data-[state=open]:bg-accent', className)}
		onclick={() => column.toggleSorting()}
	>
		{title}
		{#if column.getIsSorted() === 'desc'}
			<ArrowDown class="ml-1 h-3.5 w-3.5" />
		{:else if column.getIsSorted() === 'asc'}
			<ArrowUp class="ml-1 h-3.5 w-3.5" />
		{:else}
			<ChevronsUpDown class="ml-1 h-3.5 w-3.5" />
		{/if}
	</Button>
{/if}
