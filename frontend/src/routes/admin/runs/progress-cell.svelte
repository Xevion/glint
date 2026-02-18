<script lang="ts">
import type { CaptureRunStatus } from '$lib/bindings';
import * as Tooltip from '$lib/components/ui/tooltip';

interface Props {
	total: number;
	completed: number;
	failed: number;
	skipped: number;
	status: CaptureRunStatus;
}

let { total, completed, failed, skipped, status }: Props = $props();

let remaining = $derived(total - completed - failed - skipped);
let isTimedOut = $derived(status === 'timed_out');
</script>

{#if total > 0}
	<Tooltip.Provider delayDuration={0} disableHoverableContent>
		<Tooltip.Root>
			<Tooltip.Trigger class="flex h-2.5 w-28 overflow-hidden rounded-full bg-muted">
				{#if completed > 0}
					<div class="bg-success" style="width: {(completed / total) * 100}%"></div>
				{/if}
				{#if failed > 0}
					<div class="bg-destructive" style="width: {(failed / total) * 100}%"></div>
				{/if}
				{#if skipped > 0}
					<div
						class="bg-muted-foreground/30"
						style="width: {(skipped / total) * 100}%"
					></div>
				{/if}
				{#if remaining > 0}
					<div
						class={isTimedOut ? 'bg-warning' : 'bg-info'}
						style="width: {(remaining / total) * 100}%"
					></div>
				{/if}
			</Tooltip.Trigger>
			<Tooltip.Content>
				<p class="text-xs">
					{completed} completed, {failed} failed, {skipped} skipped, {remaining} remaining
				</p>
			</Tooltip.Content>
		</Tooltip.Root>
	</Tooltip.Provider>
{:else}
	<span class="text-muted-foreground">&mdash;</span>
{/if}
