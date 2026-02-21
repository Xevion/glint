<script lang="ts">
import { Button } from '$lib/components/ui/button';
import * as Popover from '$lib/components/ui/popover';
import { cn } from '$lib/utils';
import { CalendarDays, X } from '@lucide/svelte';

interface Preset {
	label: string;
	/** ISO duration shorthand for computing the "after" date, e.g., '24h', '7d', '30d'. */
	duration: string;
}

interface Props {
	/** Display label (e.g., "Captured"). */
	label?: string;
	/** Current "after" value — ISO date string or null. */
	after: string | null;
	/** Current "before" value — ISO date string or null. */
	before: string | null;
	class?: string;
}

let {
	label = 'Date',
	after = $bindable(null),
	before = $bindable(null),
	class: className
}: Props = $props();

let open = $state(false);
let customMode = $state(false);

const presets: Preset[] = [
	{ label: 'Last 24 hours', duration: '24h' },
	{ label: 'Last 7 days', duration: '7d' },
	{ label: 'Last 30 days', duration: '30d' },
	{ label: 'Last 90 days', duration: '90d' }
];

function durationToMs(d: string): number {
	const num = parseInt(d);
	if (d.endsWith('h')) return num * 60 * 60 * 1000;
	if (d.endsWith('d')) return num * 24 * 60 * 60 * 1000;
	return num;
}

function applyPreset(preset: Preset) {
	const now = new Date();
	const cutoff = new Date(now.getTime() - durationToMs(preset.duration));
	after = cutoff.toISOString();
	before = null;
	open = false;
	customMode = false;
}

function clear() {
	after = null;
	before = null;
	customMode = false;
}

const hasValue = $derived(after !== null || before !== null);

/** Active preset label if one matches, otherwise a date range summary. */
const displayLabel = $derived.by(() => {
	if (!hasValue) return label;
	if (before === null && after !== null) {
		// Check if it matches a preset (within 1 minute tolerance)
		const afterMs = new Date(after).getTime();
		const nowMs = Date.now();
		for (const p of presets) {
			const expected = nowMs - durationToMs(p.duration);
			if (Math.abs(afterMs - expected) < 60_000) return p.label;
		}
		return `After ${formatDate(after)}`;
	}
	if (after !== null && before !== null) {
		return `${formatDate(after)} – ${formatDate(before)}`;
	}
	if (before !== null) return `Before ${formatDate(before)}`;
	return label;
});

function formatDate(iso: string): string {
	return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

// Custom date inputs
let customAfter = $state('');
let customBefore = $state('');

/** Converts an ISO date string to a YYYY-MM-DD input value. */
function toDateInput(iso: string): string {
	return new Date(iso).toISOString().slice(0, 10);
}

function handleOpenChange(isOpen: boolean) {
	if (isOpen) {
		// Pre-fill custom inputs from active filter values
		customAfter = after ? toDateInput(after) : '';
		customBefore = before ? toDateInput(before) : '';
	} else {
		// Reset to preset view when popover closes
		customMode = false;
	}
}

function applyCustom() {
	after = customAfter ? new Date(customAfter).toISOString() : null;
	before = customBefore ? new Date(customBefore + 'T23:59:59').toISOString() : null;
	open = false;
}
</script>

<Popover.Root bind:open onOpenChange={handleOpenChange}>
	<Popover.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="outline"
				size="sm"
				class={cn('h-8 gap-1.5 border-dashed font-normal', hasValue && 'border-primary/50', className)}
			>
				<CalendarDays class="size-3.5 opacity-50" />
				{displayLabel}
				{#if hasValue}
					<span
						role="presentation"
						class="ml-0.5 rounded-full p-0.5 hover:bg-accent"
						onclick={(e) => {
							e.stopPropagation();
							clear();
							open = false;
						}}
					>
						<X class="size-3" />
					</span>
				{/if}
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-56 p-2" align="start">
		{#if !customMode}
			<div class="flex flex-col gap-0.5">
				{#each presets as preset (preset.duration)}
					<button
						type="button"
						class="rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
						onclick={() => applyPreset(preset)}
					>
						{preset.label}
					</button>
				{/each}
				<div class="my-1 border-t"></div>
				<button
					type="button"
					class="rounded-md px-2 py-1.5 text-left text-sm text-muted-foreground hover:bg-accent hover:text-foreground"
					onclick={() => (customMode = true)}
				>
					Custom range...
				</button>
			</div>
		{:else}
			<div class="flex flex-col gap-2">
				<label class="flex flex-col gap-1 text-xs font-medium">
					From
					<input
						type="date"
						class="rounded-md border bg-transparent px-2 py-1 text-sm"
						bind:value={customAfter}
					/>
				</label>
				<label class="flex flex-col gap-1 text-xs font-medium">
					To
					<input
						type="date"
						class="rounded-md border bg-transparent px-2 py-1 text-sm"
						bind:value={customBefore}
					/>
				</label>
				<div class="flex gap-1">
					<Button
						variant="ghost"
						size="sm"
						class="h-7 flex-1 text-xs"
						onclick={() => (customMode = false)}
					>
						Back
					</Button>
					<Button size="sm" class="h-7 flex-1 text-xs" onclick={applyCustom}>Apply</Button>
				</div>
			</div>
		{/if}
		{#if hasValue && !customMode}
			<div class="mt-1 border-t pt-1">
				<button
					type="button"
					class="flex w-full items-center justify-center gap-1 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
					onclick={() => {
						clear();
						open = false;
					}}
				>
					<X class="size-3" />
					Clear
				</button>
			</div>
		{/if}
	</Popover.Content>
</Popover.Root>
