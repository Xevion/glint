<script lang="ts">
import { getDimensionDisplayName } from '$lib/utils/format';

interface Props {
	dimension: string;
}

let { dimension }: Props = $props();
</script>

<span class="text-sm text-muted-foreground">{getDimensionDisplayName(dimension)}</span>
