<script lang="ts">
import type { User } from '$lib/bindings';

interface Props {
	user: User;
}

let { user }: Props = $props();
</script>

{#if user.discord_avatar}
	<img
		src="https://cdn.discordapp.com/avatars/{user.discord_id}/{user.discord_avatar}.png?size=40"
		alt="Avatar"
		class="h-8 w-8 rounded-full"
	/>
{:else}
	<div class="flex h-8 w-8 items-center justify-center rounded-full bg-muted text-xs">?</div>
{/if}
