<script lang="ts">
import { onMount } from 'svelte';
import { fly, fade } from 'svelte/transition';
import { createApiClient } from '$lib/api';
import { ApiErrorType } from '$lib/api/errors';
import { Button } from '$lib/components/ui/button';
import { resolve } from '$app/paths';
import { Monitor, CheckCircle, XCircle, Clock, Loader2 } from 'lucide-svelte';
import type { DeviceCodeStatus } from '$lib/bindings';

interface Props {
	data: { userCode: string | null };
}

let { data }: Props = $props();

type PageState =
	| { type: 'loading' }
	| { type: 'not_logged_in' }
	| { type: 'invalid_code'; message: string }
	| { type: 'expired' }
	| { type: 'already_authorized' }
	| { type: 'confirm'; code: DeviceCodeStatus }
	| { type: 'confirming' }
	| { type: 'success' }
	| { type: 'error'; message: string };

let state = $state<PageState>({ type: 'loading' });

const api = createApiClient();

async function loadCodeStatus() {
	if (!data.userCode) {
		state = { type: 'invalid_code', message: 'No device code provided' };
		return;
	}

	const result = await api.device.getCodeStatus(data.userCode);

	result.match({
		Ok: (codeStatus) => {
			if (codeStatus.status === 'authorized' || codeStatus.status === 'used') {
				state = { type: 'already_authorized' };
			} else {
				state = { type: 'confirm', code: codeStatus };
			}
		},
		Err: (err) => {
			if (err.type === ApiErrorType.Unauthorized) {
				state = { type: 'not_logged_in' };
			} else if (err.type === ApiErrorType.NotFound) {
				state = { type: 'invalid_code', message: 'Device code not found or has expired' };
			} else if (err.statusCode === 410) {
				state = { type: 'expired' };
			} else {
				state = { type: 'error', message: err.message };
			}
		}
	});
}

async function confirmAuthorization() {
	if (state.type !== 'confirm' || !data.userCode) return;

	state = { type: 'confirming' };

	const result = await api.device.confirm(data.userCode);

	result.match({
		Ok: () => {
			state = { type: 'success' };
		},
		Err: (err) => {
			if (err.type === ApiErrorType.Unauthorized) {
				state = { type: 'not_logged_in' };
			} else if (err.type === ApiErrorType.NotFound) {
				state = { type: 'expired' };
			} else {
				state = { type: 'error', message: err.message };
			}
		}
	});
}

// Build login URL with redirect back to this page
const loginUrl = $derived.by(() => {
	const currentUrl = typeof window !== 'undefined' ? window.location.href : '';
	return resolve('/api/auth/discord', { redirect: currentUrl });
});

onMount(() => {
	void loadCodeStatus();
});
</script>

<svelte:head>
	<title>Authorize Device | Glint</title>
</svelte:head>

<div class="flex min-h-[60vh] items-center justify-center">
	<div
		in:fly={{ y: 20, duration: 400 }}
		class="w-full max-w-md rounded-xl border border-border bg-card p-8"
	>
		{#if state.type === 'loading'}
			<div class="text-center" in:fade>
				<Loader2 class="mx-auto mb-4 h-12 w-12 animate-spin text-muted-foreground" />
				<p class="text-muted-foreground">Loading...</p>
			</div>
		{:else if state.type === 'not_logged_in'}
			<div class="text-center" in:fade>
				<Monitor class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Authorize Device</h1>
				<p class="mb-6 text-muted-foreground">You need to log in to authorize this device.</p>
				<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
				<Button href={loginUrl} class="w-full">Log in with Discord</Button>
			</div>
		{:else if state.type === 'invalid_code'}
			<div class="text-center" in:fade>
				<XCircle class="mx-auto mb-4 h-12 w-12 text-destructive" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Invalid Code</h1>
				<p class="mb-6 text-muted-foreground">{state.message}</p>
				<p class="text-sm text-muted-foreground">
					Make sure you entered the code correctly in your Minecraft client.
				</p>
			</div>
		{:else if state.type === 'expired'}
			<div class="text-center" in:fade>
				<Clock class="mx-auto mb-4 h-12 w-12 text-amber-500" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Code Expired</h1>
				<p class="mb-6 text-muted-foreground">
					This device code has expired. Please start the authorization process again in your
					Minecraft client.
				</p>
			</div>
		{:else if state.type === 'already_authorized'}
			<div class="text-center" in:fade>
				<CheckCircle class="mx-auto mb-4 h-12 w-12 text-green-500" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Already Authorized</h1>
				<p class="text-muted-foreground">
					This device has already been authorized. You can close this window.
				</p>
			</div>
		{:else if state.type === 'confirm'}
			<div class="text-center" in:fade>
				<Monitor class="mx-auto mb-4 h-12 w-12 text-primary" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Authorize Device</h1>
				<p class="mb-6 text-muted-foreground">
					A Minecraft client is requesting access to your Glint account.
				</p>

				<div class="mb-6 rounded-lg bg-muted p-4">
					<div class="text-xs font-medium text-muted-foreground">Device Code</div>
					<div class="font-mono text-2xl font-bold text-foreground">{state.code.user_code}</div>
				</div>

				<Button onclick={confirmAuthorization} class="w-full">Authorize This Device</Button>

				<p class="mt-4 text-xs text-muted-foreground">
					Only authorize devices you trust. This will allow the Minecraft mod to upload captures on
					your behalf.
				</p>
			</div>
		{:else if state.type === 'confirming'}
			<div class="text-center" in:fade>
				<Loader2 class="mx-auto mb-4 h-12 w-12 animate-spin text-primary" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Authorizing...</h1>
				<p class="text-muted-foreground">Please wait while we authorize your device.</p>
			</div>
		{:else if state.type === 'success'}
			<div class="text-center" in:fade>
				<CheckCircle class="mx-auto mb-4 h-12 w-12 text-green-500" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Device Authorized!</h1>
				<p class="mb-4 text-muted-foreground">
					Your Minecraft client has been authorized. You can close this window and return to
					Minecraft.
				</p>
				<p class="text-sm text-muted-foreground">The mod will automatically continue setup.</p>
			</div>
		{:else if state.type === 'error'}
			<div class="text-center" in:fade>
				<XCircle class="mx-auto mb-4 h-12 w-12 text-destructive" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Error</h1>
				<p class="mb-6 text-muted-foreground">{state.message}</p>
				<Button onclick={loadCodeStatus} variant="outline">Try Again</Button>
			</div>
		{/if}
	</div>
</div>
