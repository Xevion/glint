<script lang="ts">
import { Button } from './button';
import * as Dialog from './dialog';

interface Props {
	open: boolean;
	title: string;
	description?: string;
	confirmLabel?: string;
	cancelLabel?: string;
	variant?: 'default' | 'destructive';
	onConfirm: () => void | Promise<void>;
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

let confirming = $state(false);

async function handleConfirm() {
	confirming = true;
	try {
		await onConfirm();
		open = false;
	} finally {
		confirming = false;
	}
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
			<Button variant="outline" onclick={handleCancel} disabled={confirming}>{cancelLabel}</Button>
			<Button {variant} onclick={handleConfirm} disabled={confirming}>
				{confirming ? 'Processing...' : confirmLabel}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
