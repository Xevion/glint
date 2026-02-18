<script lang="ts">
import { formatDuration } from '$lib/utils/format';

interface Props {
	started_at: string;
	completed_at?: string | null;
}

let { started_at, completed_at }: Props = $props();
</script>

{formatDuration({ started_at, completed_at })}
