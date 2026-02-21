<script lang="ts">
import type { AdminCaptureNode } from './queries';

interface Props {
	capture: AdminCaptureNode;
}

let { capture }: Props = $props();
</script>

<div>
	<a
		href="/shaders/{capture.shaderSlug}"
		class="relative z-10 font-medium text-primary hover:underline"
		>{capture.shaderName}</a
	>
	<div class="text-xs text-muted-foreground">{capture.shaderVersion}</div>
</div>
