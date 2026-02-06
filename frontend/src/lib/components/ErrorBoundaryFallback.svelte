<script lang="ts">
import { Button } from '$lib/components/ui/button';
import * as Collapsible from '$lib/components/ui/collapsible';
import { ChevronDown, CircleAlert } from '@lucide/svelte';

interface Props {
	error: Error;
	reset: () => void;
	title?: string;
}

let { error, reset, title = 'Something went wrong' }: Props = $props();

let detailsOpen = $state(false);
</script>

<div class="flex min-h-[200px] flex-col items-center justify-center gap-4 p-6 text-center">
	<div class="flex flex-col items-center gap-2">
		<CircleAlert class="h-12 w-12 text-destructive" />
		<h2 class="text-lg font-semibold text-foreground">{title}</h2>
		<p class="text-sm text-muted-foreground">{error.message}</p>
	</div>

	<Button variant="outline" onclick={reset}>
		Reload
	</Button>

	{#if error.stack}
		<Collapsible.Root bind:open={detailsOpen} class="w-full max-w-lg">
			<Collapsible.Trigger
				class="flex w-full items-center justify-center gap-1 text-sm text-muted-foreground hover:text-foreground"
			>
				<span>Details</span>
				<ChevronDown
					class="h-4 w-4 transition-transform duration-200 {detailsOpen ? 'rotate-180' : ''}"
				/>
			</Collapsible.Trigger>
			<Collapsible.Content>
				<pre
					class="mt-2 max-h-48 overflow-auto rounded-md bg-muted p-3 text-left text-xs text-muted-foreground">{error.stack}</pre>
			</Collapsible.Content>
		</Collapsible.Root>
	{/if}
</div>
