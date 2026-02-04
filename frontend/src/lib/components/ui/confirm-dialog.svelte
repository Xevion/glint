<script lang="ts">
import * as Dialog from './dialog';
import { Button } from './button';

interface Props {
	open: boolean;
	title: string;
	description?: string;
	confirmLabel?: string;
	cancelLabel?: string;
	variant?: 'default' | 'destructive';
	onConfirm: () => void;
	onCancel?: () => void;
}

let {
	open = $bindable(false),
	title,
	description,
	confirmLabel = 'Confirm',
	cancelLabel = 'Cancel',
	variant = 'destructive',
	onConfirm,
	onCancel
}: Props = $props();

function handleConfirm() {
	open = false;
	onConfirm();
}

function handleCancel() {
	open = false;
	onCancel?.();
}
</script>

<Dialog.Root bind:open>
	<Dialog.Content showCloseButton={false} class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{title}</Dialog.Title>
			{#if description}
				<Dialog.Description>{description}</Dialog.Description>
			{/if}
		</Dialog.Header>
		<Dialog.Footer class="mt-4">
			<Button variant="outline" onclick={handleCancel}>{cancelLabel}</Button>
			<Button {variant} onclick={handleConfirm}>{confirmLabel}</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
