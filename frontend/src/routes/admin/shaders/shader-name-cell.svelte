<script lang="ts">
import type { AdminShader } from './queries';

interface Props {
	shader: AdminShader;
}

let { shader }: Props = $props();
</script>

<span class="font-medium">{shader.name}</span>
