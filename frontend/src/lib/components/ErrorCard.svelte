<script lang="ts">
import { untrack, type Snippet } from 'svelte';
import { dev } from '$app/environment';
import { Button } from '$lib/components/ui/button';
import * as Collapsible from '$lib/components/ui/collapsible';
import { cn } from '$lib/utils';
import { formatTime } from '$lib/utils/format';
import { ChevronDown, Clipboard, Check } from '@lucide/svelte';

interface Props {
	title: string;
	description?: string;
	statusLabel?: string;
	message?: string;
	errorId?: string;
	requestId?: string;
	timestamp?: string;
	stack?: string;
	code?: string;
	detail?: string;
	defaultStackOpen?: boolean;
	actions?: Snippet;
	class?: string;
}

let {
	title,
	description,
	statusLabel,
	message,
	errorId,
	requestId,
	timestamp,
	stack,
	code,
	detail,
	defaultStackOpen = dev,
	actions,
	class: className
}: Props = $props();

let stackOpen = $state(untrack(() => defaultStackOpen));
let copied = $state(false);
let idCopied = $state(false);

/** Show description only as a fallback when there's no message. */
let bodyText = $derived(message ?? description);

let metadataItems = $derived.by(() => {
	const items: { label: string; value: string }[] = [];
	if (code) items.push({ label: 'Code', value: code });
	if (timestamp) items.push({ label: 'Time', value: formatTime(timestamp) });
	if (requestId) items.push({ label: 'Request', value: requestId });
	return items;
});

function buildErrorReport(): string {
	const lines: string[] = [];
	if (errorId) lines.push(`Error ID: ${errorId}`);
	if (statusLabel) lines.push(`Status: ${statusLabel}`);
	if (code) lines.push(`Code: ${code}`);
	lines.push(`Title: ${title}`);
	if (message) lines.push(`Message: ${message}`);
	if (detail) lines.push(`Detail: ${detail}`);
	if (timestamp) lines.push(`Timestamp: ${timestamp}`);
	if (requestId) lines.push(`Request ID: ${requestId}`);
	if (typeof window !== 'undefined') lines.push(`URL: ${window.location.href}`);
	if (stack) lines.push('', 'Stack Trace:', stack);
	return lines.join('\n');
}

async function copyToClipboard(text: string) {
	try {
		await navigator.clipboard.writeText(text);
	} catch {
		const textArea = document.createElement('textarea');
		textArea.value = text;
		textArea.style.position = 'fixed';
		textArea.style.opacity = '0';
		document.body.appendChild(textArea);
		textArea.select();
		document.execCommand('copy');
		document.body.removeChild(textArea);
	}
}

async function copyErrorDetails() {
	await copyToClipboard(buildErrorReport());
	copied = true;
	setTimeout(() => (copied = false), 2000);
}

async function copyErrorId() {
	if (!errorId) return;
	await copyToClipboard(errorId);
	idCopied = true;
	setTimeout(() => (idCopied = false), 2000);
}
</script>

<div class={cn('rounded-lg border bg-card text-card-foreground shadow-sm', className)}>
	<!-- Header -->
	<div class="flex items-center gap-3 border-b px-6 py-2.5">
		<h2 class="text-lg font-semibold">{title}</h2>
		{#if statusLabel}
			<span class="ml-auto font-mono text-2xl font-bold tabular-nums text-muted-foreground">
				{statusLabel}
			</span>
		{/if}
	</div>

	<!-- Body -->
	<div class="space-y-3 px-6 py-5 text-sm text-muted-foreground">
		{#if bodyText}
			<p>{bodyText}</p>
		{/if}

		{#if detail}
			<pre class="whitespace-pre-wrap font-mono text-xs leading-relaxed">{detail}</pre>
		{/if}

		{#if metadataItems.length > 0}
			<div class="flex flex-wrap gap-x-4 gap-y-1 font-mono text-xs">
				{#each metadataItems as item (item.label)}
					<span><span class="opacity-50">{item.label}:</span> {item.value}</span>
				{/each}
			</div>
		{/if}

		{#if stack}
			<Collapsible.Root bind:open={stackOpen}>
				<Collapsible.Trigger
					class="flex cursor-pointer items-center gap-1.5 text-sm transition-colors hover:text-foreground"
				>
					<ChevronDown
						class="h-4 w-4 transition-transform duration-200 {stackOpen
							? 'rotate-180'
							: ''}"
					/>
					<span>Stack Trace</span>
				</Collapsible.Trigger>
				<Collapsible.Content>
					<pre
						class="mt-2 max-h-80 overflow-auto rounded-md border bg-muted/50 p-3 font-mono text-xs leading-relaxed">{stack}</pre>
				</Collapsible.Content>
			</Collapsible.Root>
		{/if}
	</div>

	<!-- Footer -->
	<div class="flex items-center gap-3 border-t px-6 py-3">
		<!-- Left: actions -->
		{#if actions}
			<div class="flex shrink-0 items-center gap-2">
				{@render actions()}
			</div>
		{/if}

		<!-- Center: error ID (clickable to copy) -->
		{#if errorId}
			<div class="min-w-0 flex-1 overflow-hidden font-mono text-xs text-muted-foreground">
				<button
					type="button"
					onclick={copyErrorId}
					class="inline-flex min-w-0 cursor-pointer items-center gap-1 transition-colors hover:text-foreground"
					title="Click to copy error ID"
				>
					<span class="truncate">{errorId}</span>
					<Check
						class="h-3 w-3 shrink-0 text-green-500 transition-opacity duration-150 {idCopied
							? 'opacity-100'
							: 'opacity-0'}"
					/>
				</button>
			</div>
		{/if}

		<!-- Right: copy details -->
		<div class="ml-auto shrink-0">
			<Button variant="outline" size="sm" onclick={copyErrorDetails}>
				{#if copied}
					<Check class="h-3.5 w-3.5" />
					Copied
				{:else}
					<Clipboard class="h-3.5 w-3.5" />
					Copy Details
				{/if}
			</Button>
		</div>
	</div>
</div>
