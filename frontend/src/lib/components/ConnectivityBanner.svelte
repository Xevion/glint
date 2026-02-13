<script lang="ts">
import { connectivity } from '$lib/stores/connectivity.svelte';
import { ServerCrash, WifiOff } from '@lucide/svelte';
import { slide } from 'svelte/transition';
</script>

{#if !connectivity.online}
	<div
		transition:slide={{ duration: 200 }}
		class="fixed inset-x-0 top-0 z-50 flex items-center justify-center gap-2 bg-warning px-4 py-2 text-sm font-medium text-warning-foreground"
	>
		<WifiOff class="h-4 w-4" />
		You appear to be offline
	</div>
{:else if !connectivity.backendReachable}
	<div
		transition:slide={{ duration: 200 }}
		class="fixed inset-x-0 top-0 z-50 flex items-center justify-center gap-2 bg-destructive px-4 py-2 text-sm font-medium text-destructive-foreground"
	>
		<ServerCrash class="h-4 w-4" />
		Backend is unreachable
	</div>
{/if}
