<script lang="ts">
import { page } from '$app/state';
import BrandIcon from '$lib/components/icons/BrandIcon.svelte';
import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { CircleAlert, LogIn } from '@lucide/svelte';
import { fly } from 'svelte/transition';

/** Maps backend OAuth error codes to user-facing messages. */
const ERROR_MESSAGES: Record<string, string> = {
	session_expired:
		'Your login session expired. This usually happens if you waited too long before authorizing, or your browser is blocking cookies.',
	state_invalid:
		'The login request was malformed. This can happen if the URL was modified or corrupted.',
	csrf_mismatch:
		"Your login session doesn't match. This can happen if you used the back button, opened multiple login tabs, or the link was stale.",
	exchange_failed:
		'Discord rejected the authorization. The link may have expired \u2014 Discord codes are only valid for a few seconds.',
	discord_error:
		"Discord's API returned an error. Discord may be experiencing issues \u2014 check their status page and try again shortly.",
	discord_unavailable:
		'Discord login is temporarily unavailable. Please try again later or contact the site administrator.',
	server_error:
		'Something went wrong on our end while completing your login. Please try again \u2014 if it persists, contact the site administrator.'
};

const FALLBACK_MESSAGE = 'Something went wrong during login. Please try again.';

const errorCode = $derived(page.url.searchParams.get('error'));
const errorMessage = $derived(errorCode ? (ERROR_MESSAGES[errorCode] ?? FALLBACK_MESSAGE) : null);

// Build Discord OAuth URL with redirect back to where the user came from
const redirectTarget = $derived(page.url.searchParams.get('redirect') ?? '/');
const discordLoginUrl = $derived(
	`/api/auth/discord?redirect=${encodeURIComponent(redirectTarget)}`
);

const pageTitle = $derived(errorCode ? 'Login Failed | Glint' : 'Sign In | Glint');
</script>

<svelte:head>
	<title>{pageTitle}</title>
</svelte:head>

<div class="flex min-h-[60vh] items-center justify-center">
	<div
		in:fly={{ y: 20, duration: 400 }}
		class="w-full max-w-sm space-y-4"
	>
		{#if errorMessage}
			<Alert variant="destructive">
				<CircleAlert />
				<AlertTitle>Login failed</AlertTitle>
				<AlertDescription>{errorMessage}</AlertDescription>
			</Alert>
		{/if}

		<div class="rounded-xl border border-border bg-card p-8">
			<div class="text-center">
				<LogIn class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Sign in to Glint</h1>
				<p class="mb-6 text-muted-foreground">
					{#if errorCode}
						Try signing in again to continue.
					{:else}
						Sign in with your Discord account to access additional features.
					{/if}
				</p>

			<Button href={discordLoginUrl} class="w-full gap-2">
				<BrandIcon name="discord" class="h-5 w-5" />
				{errorCode ? 'Try again with Discord' : 'Sign in with Discord'}
			</Button>
			</div>
		</div>
	</div>
</div>
