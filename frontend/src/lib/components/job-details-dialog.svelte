<script lang="ts">
	import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
	import TimeAgo from '$lib/components/time-ago.svelte';
	import type { JobWithDetails } from '$lib/api/endpoints/admin';

	interface Props {
		job: JobWithDetails | null;
		open?: boolean;
	}

	let { job, open = $bindable(false) }: Props = $props();

	function parseSceneIds(sceneIds: string | null): string[] {
		if (!sceneIds) return [];
		try {
			return JSON.parse(sceneIds) as string[];
		} catch {
			return [];
		}
	}

	function parseProfiles(profiles: string | null): string[] {
		if (!profiles) return [];
		try {
			return JSON.parse(profiles) as string[];
		} catch {
			return [];
		}
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-w-2xl">
		<DialogHeader>
			<DialogTitle>Job Details</DialogTitle>
		</DialogHeader>

		{#if job}
			<div class="grid gap-4">
				<div class="grid grid-cols-2 gap-4">
					<div>
						<h3 class="mb-2 text-sm font-semibold">Job ID</h3>
						<p class="font-mono text-xs text-muted-foreground">{job.id}</p>
					</div>
					<div>
						<h3 class="mb-2 text-sm font-semibold">Status</h3>
						<p class="capitalize">{job.status}</p>
					</div>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div>
						<h3 class="mb-2 text-sm font-semibold">Shader</h3>
						<p>{job.shader_name}</p>
						<p class="text-xs text-muted-foreground">Version: {job.shader_version}</p>
					</div>
					<div>
						<h3 class="mb-2 text-sm font-semibold">Priority</h3>
						<p>{job.priority}</p>
					</div>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div>
						<h3 class="mb-2 text-sm font-semibold">Attempts</h3>
						<p>{job.attempts} / {job.max_attempts}</p>
					</div>
					<div>
						<h3 class="mb-2 text-sm font-semibold">Agent ID</h3>
						<p class="font-mono text-xs text-muted-foreground">
							{job.agent_id ?? 'Not claimed'}
						</p>
					</div>
				</div>

				<div>
					<h3 class="mb-2 text-sm font-semibold">Scene IDs ({job.scene_count})</h3>
					<div class="rounded-md border bg-muted p-2">
						{#if parseSceneIds(job.scene_ids).length > 0}
							<ul class="space-y-1">
								{#each parseSceneIds(job.scene_ids) as sceneId (sceneId)}
									<li class="font-mono text-xs">{sceneId}</li>
								{/each}
							</ul>
						{:else}
							<p class="text-xs text-muted-foreground">No scenes</p>
						{/if}
					</div>
				</div>

				{#if job.profiles}
					<div>
						<h3 class="mb-2 text-sm font-semibold">Profiles</h3>
						<div class="rounded-md border bg-muted p-2">
							<ul class="space-y-1">
								{#each parseProfiles(job.profiles) as profile (profile)}
									<li class="font-mono text-xs">{profile}</li>
								{/each}
							</ul>
						</div>
					</div>
				{/if}

				<div class="grid grid-cols-2 gap-4">
					<div>
						<h3 class="mb-2 text-sm font-semibold">Created</h3>
						<TimeAgo timestamp={new Date(job.created_at)} />
					</div>
					{#if job.claimed_at}
						<div>
							<h3 class="mb-2 text-sm font-semibold">Claimed</h3>
							<TimeAgo timestamp={new Date(job.claimed_at)} />
						</div>
					{/if}
				</div>

				<div class="grid grid-cols-2 gap-4">
					{#if job.started_at}
						<div>
							<h3 class="mb-2 text-sm font-semibold">Started</h3>
							<TimeAgo timestamp={new Date(job.started_at)} />
						</div>
					{/if}
					{#if job.completed_at}
						<div>
							<h3 class="mb-2 text-sm font-semibold">Completed</h3>
							<TimeAgo timestamp={new Date(job.completed_at)} />
						</div>
					{/if}
				</div>

				{#if job.last_heartbeat}
					<div>
						<h3 class="mb-2 text-sm font-semibold">Last Heartbeat</h3>
						<TimeAgo timestamp={new Date(job.last_heartbeat)} />
					</div>
				{/if}

				{#if job.error_message}
					<div>
						<h3 class="mb-2 text-sm font-semibold">Error Message</h3>
						<div class="rounded-md border border-destructive bg-destructive/10 p-3">
							<p class="text-sm text-destructive">{job.error_message}</p>
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</DialogContent>
</Dialog>
