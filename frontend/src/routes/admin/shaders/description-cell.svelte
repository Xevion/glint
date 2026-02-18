<script lang="ts">
interface Props {
	description: string | undefined;
}

let { description }: Props = $props();
</script>

{#if description}
	<span class="block max-w-56 truncate">{description}</span>
{:else}
	<span class="text-muted-foreground">-</span>
{/if}
