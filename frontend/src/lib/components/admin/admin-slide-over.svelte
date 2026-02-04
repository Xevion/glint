<script lang="ts">
import * as Sheet from '$lib/components/ui/sheet';
import type { Snippet } from 'svelte';

interface Props {
	open: boolean;
	onClose: () => void;
	title: string;
	description?: string;
	width?: 'default' | 'wide';
	children: Snippet;
	footer?: Snippet;
}

let { open, onClose, title, description, width = 'default', children, footer }: Props = $props();

function handleOpenChange(value: boolean) {
	if (!value) {
		onClose();
	}
}
</script>

<Sheet.Root {open} onOpenChange={handleOpenChange}>
	<Sheet.Content
		side="right"
		class={width === 'wide' ? 'sm:max-w-lg' : 'sm:max-w-md'}
	>
		<Sheet.Header>
			<Sheet.Title>{title}</Sheet.Title>
			{#if description}
				<Sheet.Description>{description}</Sheet.Description>
			{/if}
		</Sheet.Header>

		<div class="flex-1 overflow-y-auto py-4">
			{@render children()}
		</div>

		{#if footer}
			<Sheet.Footer>
				{@render footer()}
			</Sheet.Footer>
		{/if}
	</Sheet.Content>
</Sheet.Root>
