<script lang="ts">
import type { Snippet } from 'svelte';
import ErrorCard from './ErrorCard.svelte';
import { Button } from '$lib/components/ui/button';

interface Props {
	title?: string;
	children: Snippet;
}

let { title, children }: Props = $props();

function handleError(error: unknown) {
	const section = title ?? 'Unknown section';
	console.error(`[SectionBoundary: ${section}]`, error);
}
</script>

<svelte:boundary onerror={handleError}>
	{@render children()}

	{#snippet failed(error, reset)}
		{@const err = error instanceof Error ? error : new Error(String(error))}
		<ErrorCard title={title ?? 'Something went wrong'} message={err.message} stack={err.stack}>
			{#snippet actions()}
				<Button variant="outline" size="sm" onclick={reset}>Try Again</Button>
			{/snippet}
		</ErrorCard>
	{/snippet}
</svelte:boundary>
