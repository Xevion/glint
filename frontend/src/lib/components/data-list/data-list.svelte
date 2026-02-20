<script lang="ts" generics="T">
import { Alert } from '$lib/components/ui/alert';
import type { Snippet } from 'svelte';

interface Props {
	/** Items to display */
	items: T[];
	/** Error message — when set, the error state is shown instead of content */
	error?: string | null;
	/** Message shown when items array is empty (default: "No items yet.") */
	emptyMessage?: string;
	/** Content snippet — receives the items array */
	content: Snippet<[T[]]>;
	/** Optional snippet rendered above the content (toolbar, filters, etc.) */
	header?: Snippet;
	/** Optional snippet rendered when there are no items (overrides default empty message) */
	empty?: Snippet;
}

let {
	items,
	error = null,
	emptyMessage = 'No items yet.',
	content,
	header,
	empty
}: Props = $props();
</script>

{@render header?.()}

{#if error}
	<Alert variant="destructive">Error: {error}</Alert>
{:else if items.length === 0}
	{#if empty}
		{@render empty()}
	{:else}
		<p class="text-foreground">{emptyMessage}</p>
	{/if}
{:else}
	{@render content(items)}
{/if}
