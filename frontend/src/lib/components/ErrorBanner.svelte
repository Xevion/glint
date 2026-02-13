<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { cn } from '$lib/utils';
import { AlertTriangle, LoaderCircle, RefreshCw } from '@lucide/svelte';

interface Props {
	message: string;
	retryable?: boolean;
	class?: string;
}

let { message, retryable = true, class: className }: Props = $props();

let retrying = $state(false);

async function retry() {
	retrying = true;
	try {
		await invalidateAll();
	} finally {
		retrying = false;
	}
}
</script>

<Alert variant="destructive" class={cn('flex items-center gap-3', className)}>
	<AlertTriangle class="h-4 w-4 shrink-0" />
	<span class="flex-1 text-sm">{message}</span>
	{#if retryable}
		<Button
			variant="outline"
			size="sm"
			class="ml-auto h-7 shrink-0 gap-1.5 border-destructive/30 text-destructive hover:bg-destructive/10 hover:text-destructive"
			disabled={retrying}
			onclick={retry}
		>
			{#if retrying}
				<LoaderCircle class="h-3.5 w-3.5 animate-spin" />
				Retrying...
			{:else}
				<RefreshCw class="h-3.5 w-3.5" />
				Retry
			{/if}
		</Button>
	{/if}
</Alert>
