<script lang="ts">
import { page } from '$app/state';
import { Button } from '$lib/components/ui/button';
import { LogIn } from '@lucide/svelte';
import { fly } from 'svelte/transition';

// Build Discord OAuth URL with redirect back to where the user came from
const redirectTarget = $derived(page.url.searchParams.get('redirect') ?? '/');
const discordLoginUrl = $derived(
	`/api/auth/discord?redirect=${encodeURIComponent(redirectTarget)}`
);
</script>

<svelte:head>
	<title>Sign In | Glint</title>
</svelte:head>

<div class="flex min-h-[60vh] items-center justify-center">
	<div
		in:fly={{ y: 20, duration: 400 }}
		class="w-full max-w-sm rounded-xl border border-border bg-card p-8"
	>
		<div class="text-center">
			<LogIn class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
			<h1 class="mb-2 text-2xl font-bold text-card-foreground">Sign in to Glint</h1>
			<p class="mb-6 text-muted-foreground">
				Sign in with your Discord account to access additional features.
			</p>

			<Button href={discordLoginUrl} class="w-full gap-2">
				<svg viewBox="0 0 24 24" class="h-5 w-5" fill="currentColor">
					<path
						d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03z"
					/>
				</svg>
				Sign in with Discord
			</Button>
		</div>
	</div>
</div>
