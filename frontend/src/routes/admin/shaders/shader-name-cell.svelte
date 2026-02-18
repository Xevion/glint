<script lang="ts">
import type { ShaderListItem } from '$lib/bindings';

interface Props {
	shader: ShaderListItem;
}

let { shader }: Props = $props();
</script>

<span class="font-medium">{shader.name}</span>
