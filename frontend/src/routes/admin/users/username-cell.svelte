<script lang="ts">
interface Props {
	username: string;
}

let { username }: Props = $props();
</script>

<span class="font-medium">{username}</span>
