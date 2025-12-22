<script lang="ts">
	import RelativeTime from 'svelte-relative-time';

	interface Props {
		timestamp: string | Date;
	}

	let { timestamp }: Props = $props();

	const date = $derived(typeof timestamp === 'string' ? new Date(timestamp) : timestamp);
	const absoluteTime = $derived(
		date.toLocaleString('en-US', {
			dateStyle: 'full',
			timeStyle: 'long'
		})
	);
</script>

<time title={absoluteTime} datetime={date.toISOString()}>
	<RelativeTime {date} locale="en" live />
</time>
