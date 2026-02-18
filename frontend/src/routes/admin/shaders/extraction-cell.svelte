<script lang="ts">
import type { ExtractionSummary } from '$lib/bindings';
import * as Tooltip from '$lib/components/ui/tooltip';
import { Check, CircleAlert, Clock } from '@lucide/svelte';

interface Props {
	versionCount: number;
	summary: ExtractionSummary | undefined;
}

let { versionCount, summary }: Props = $props();

let ratio = $derived(summary && summary.total > 0 ? summary.completed / summary.total : 0);
let status = $derived.by(() => {
	if (!summary || summary.total === 0) return 'none' as const;
	if (ratio >= 1) return 'complete' as const;
	if (ratio < 0.1) return 'failing' as const;
	return 'partial' as const;
});
</script>

{#if versionCount === 0}
	<span class="text-sm text-muted-foreground">0</span>
{:else}
	<Tooltip.Provider delayDuration={0} disableHoverableContent>
		<Tooltip.Root>
			<Tooltip.Trigger class="flex items-center gap-1.5">
				<span class="inline-flex size-4 shrink-0 items-center justify-center">
					{#if status === 'complete'}
						<Check class="size-4 text-emerald-500" />
					{:else if status === 'failing'}
						<CircleAlert class="size-4 text-red-500" />
					{:else if status === 'partial'}
						<Clock class="size-4 text-amber-500" />
					{/if}
				</span>
				<span class="text-sm tabular-nums">{versionCount}</span>
			</Tooltip.Trigger>
			{#if summary && summary.total > 0}
				<Tooltip.Content>
					<p class="text-xs">
						{summary.completed} extracted, {summary.failed} failed, {summary.pending} pending, {summary.skipped} skipped
					</p>
				</Tooltip.Content>
			{/if}
		</Tooltip.Root>
	</Tooltip.Provider>
{/if}
