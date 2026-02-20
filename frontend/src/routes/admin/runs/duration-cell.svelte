<script lang="ts">
import { formatElapsedDuration } from '$lib/utils/format';

interface Props {
	started_at: string;
	completed_at?: string | null;
}

let { started_at, completed_at }: Props = $props();
</script>

{formatElapsedDuration({ started_at, completed_at })}
