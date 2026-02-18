<script lang="ts">
import { formatBytes } from '$lib/utils/format';
import { Download } from '@lucide/svelte';

interface Props {
	fileSize?: number;
	fileHash?: string;
	downloadUrl?: string;
}

let { fileSize, fileHash, downloadUrl }: Props = $props();

let hasAny = $derived(fileSize != null || fileHash != null || downloadUrl != null);
</script>

{#if hasAny}
	<div class="flex items-center gap-2 text-xs text-muted-foreground">
		{#if fileSize != null}
			<span>{formatBytes(fileSize)}</span>
		{/if}
		{#if fileHash}
			<span class="font-mono text-[10px]" title={fileHash}>{fileHash.slice(0, 8)}</span>
		{/if}
		{#if downloadUrl}
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
			<a
				href={downloadUrl}
				target="_blank"
				rel="noopener noreferrer"
				class="inline-flex items-center hover:text-foreground"
				title="Download"
				onclick={(e) => e.stopPropagation()}
			>
				<Download class="size-3.5" />
			</a>
		{/if}
	</div>
{:else}
	<span class="text-muted-foreground">-</span>
{/if}
