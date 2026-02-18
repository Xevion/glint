<script lang="ts">
import type { ExtractionStatus } from '$lib/bindings';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import * as Tooltip from '$lib/components/ui/tooltip';
import { AlertTriangle, Check, Clock, SkipForward } from '@lucide/svelte';

interface Props {
	status: ExtractionStatus;
	extractedAt?: string;
	error?: string;
}

let { status, extractedAt, error: extractionError }: Props = $props();

const labels: Record<ExtractionStatus, string> = {
	completed: 'Extracted',
	failed: 'Failed',
	pending: 'Pending',
	skipped: 'Skipped'
};

const iconColors: Record<ExtractionStatus, string> = {
	completed: 'text-emerald-500',
	failed: 'text-red-500',
	pending: 'text-amber-500',
	skipped: 'text-muted-foreground'
};

let hasTooltip = $derived(!!extractedAt || !!extractionError);
</script>

{#snippet content()}
	<span class="inline-flex size-4 shrink-0 items-center justify-center">
		{#if status === 'completed'}
			<Check class="size-4 {iconColors[status]}" />
		{:else if status === 'failed'}
			<AlertTriangle class="size-4 {iconColors[status]}" />
		{:else if status === 'pending'}
			<Clock class="size-4 {iconColors[status]}" />
		{:else}
			<SkipForward class="size-4 {iconColors[status]}" />
		{/if}
	</span>
	<span class="text-sm">{labels[status]}</span>
{/snippet}

{#if hasTooltip}
	<Tooltip.Provider delayDuration={0} disableHoverableContent>
		<Tooltip.Root>
			<Tooltip.Trigger class="flex items-center gap-1.5">
				{@render content()}
			</Tooltip.Trigger>
			<Tooltip.Content>
				<div class="space-y-1 text-xs">
					{#if extractedAt}
						<p><TimeAgo timestamp={extractedAt} /></p>
					{/if}
					{#if extractionError}
						<p class="max-w-64 font-mono text-destructive">{extractionError}</p>
					{/if}
				</div>
			</Tooltip.Content>
		</Tooltip.Root>
	</Tooltip.Provider>
{:else}
	<div class="flex items-center gap-1.5">
		{@render content()}
	</div>
{/if}
