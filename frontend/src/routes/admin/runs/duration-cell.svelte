<script lang="ts">
import { formatDuration } from '$lib/utils/format';

interface Props {
	durationSecs: number | null;
}

let { durationSecs }: Props = $props();
</script>

{#if durationSecs != null}
	{formatDuration(durationSecs * 1000)}
{:else}
	In progress
{/if}
