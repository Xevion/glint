<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import StatusBadge from './status-badge.svelte';

const { Story } = defineMeta({
	title: 'UI/StatusBadge',
	component: StatusBadge,
	tags: ['autodocs']
});
</script>

<Story name="Active" args={{ status: 'active' }}>
	{#snippet children(args)}
		<StatusBadge {...args}>Active</StatusBadge>
	{/snippet}
</Story>

<Story name="Inactive" args={{ status: 'inactive' }}>
	{#snippet children(args)}
		<StatusBadge {...args}>Inactive</StatusBadge>
	{/snippet}
</Story>

<Story name="Pending" args={{ status: 'pending' }}>
	{#snippet children(args)}
		<StatusBadge {...args}>Pending</StatusBadge>
	{/snippet}
</Story>

<Story name="Error" args={{ status: 'error' }}>
	{#snippet children(args)}
		<StatusBadge {...args}>Error</StatusBadge>
	{/snippet}
</Story>

<Story name="Info" args={{ status: 'info' }}>
	{#snippet children(args)}
		<StatusBadge {...args}>Info</StatusBadge>
	{/snippet}
</Story>

<Story name="All Statuses">
	{#snippet children(_args)}
		<div class="flex flex-wrap gap-2">
			<StatusBadge status="active">Active</StatusBadge>
			<StatusBadge status="inactive">Inactive</StatusBadge>
			<StatusBadge status="pending">Pending</StatusBadge>
			<StatusBadge status="error">Error</StatusBadge>
			<StatusBadge status="info">Info</StatusBadge>
		</div>
	{/snippet}
</Story>

<Story name="Long Text" args={{ status: 'active' }}>
	{#snippet children(args)}
		<StatusBadge {...args}>Minecraft 1.21.4</StatusBadge>
	{/snippet}
</Story>
