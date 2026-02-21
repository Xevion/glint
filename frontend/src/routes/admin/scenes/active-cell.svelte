<script lang="ts">
interface Props {
	active: boolean;
}

let { active }: Props = $props();
</script>

{#if active}
	<span class="rounded bg-emerald-500/10 px-1.5 py-0.5 text-xs font-medium text-emerald-600 dark:text-emerald-400">Active</span>
{:else}
	<span class="rounded bg-destructive/10 px-1.5 py-0.5 text-xs font-medium text-destructive">Inactive</span>
{/if}
