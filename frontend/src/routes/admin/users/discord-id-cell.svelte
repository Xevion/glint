<script lang="ts">
interface Props {
	discordId: string;
}

let { discordId }: Props = $props();
</script>

<code class="text-xs text-muted-foreground">{discordId}</code>
