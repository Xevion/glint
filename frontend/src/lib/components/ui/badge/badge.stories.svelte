<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Badge from './badge.svelte';

const { Story } = defineMeta({
	title: 'UI/Badge',
	component: Badge,
	tags: ['autodocs']
});
</script>

<Story name="Default" args={{ variant: 'default' }}>
	{#snippet children(args)}
		<Badge {...args}>Badge</Badge>
	{/snippet}
</Story>

<Story name="Secondary" args={{ variant: 'secondary' }}>
	{#snippet children(args)}
		<Badge {...args}>Secondary</Badge>
	{/snippet}
</Story>

<Story name="Destructive" args={{ variant: 'destructive' }}>
	{#snippet children(args)}
		<Badge {...args}>Destructive</Badge>
	{/snippet}
</Story>

<Story name="Outline" args={{ variant: 'outline' }}>
	{#snippet children(args)}
		<Badge {...args}>Outline</Badge>
	{/snippet}
</Story>

<Story name="As Link" args={{ variant: 'default', href: '#' }}>
	{#snippet children(args)}
		<Badge {...args}>Clickable</Badge>
	{/snippet}
</Story>

<Story name="All Variants">
	{#snippet children(_args)}
		<div class="flex flex-wrap gap-2">
			<Badge variant="default">Default</Badge>
			<Badge variant="secondary">Secondary</Badge>
			<Badge variant="destructive">Destructive</Badge>
			<Badge variant="outline">Outline</Badge>
		</div>
	{/snippet}
</Story>
