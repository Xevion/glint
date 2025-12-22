<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Plus, LoaderCircle, CircleAlert } from '@lucide/svelte';
	import { api } from '$lib/api';
	import type { Shader, ShaderVersion, Scene } from '$lib/api/types';
	import type { CreateJobRequest } from '$lib/api/endpoints/admin';
	import { SvelteSet } from 'svelte/reactivity';

	interface Props {
		onJobCreated?: () => void;
	}

	let { onJobCreated }: Props = $props();

	// Dialog state
	let open = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);

	// Data
	let shaders = $state<Shader[]>([]);
	let shaderVersions = $state<Map<string, ShaderVersion[]>>(new Map());
	let scenes = $state<Scene[]>([]);

	// Form state
	let selectedShaderId = $state<string>('');
	let selectedVersionId = $state<string>('');
	let selectedSceneIds = new SvelteSet<string>();
	let profilesInput = $state<string>('');
	let priority = $state<number>(0);

	// Load data when dialog opens
	$effect(() => {
		if (open && shaders.length === 0) {
			void loadData();
		}
	});

	async function loadData() {
		loading = true;
		error = null;

		try {
			// Load shaders and scenes
			const [shadersResult, scenesResult] = await Promise.all([
				api.shaders.list(),
				api.scenes.list()
			]);

			if (shadersResult.isOk) {
				shaders = shadersResult.value;

				// Load versions for each shader
				for (const shader of shaders) {
					const shaderDetail = await api.shaders.getBySlug(shader.slug);
					if (shaderDetail.isOk) {
						shaderVersions.set(shader.id, shaderDetail.value.versions);
					}
				}
			} else {
				error = 'Failed to load shaders: ' + shadersResult.error.message;
			}

			if (scenesResult.isOk) {
				scenes = scenesResult.value;
			} else {
				error = 'Failed to load scenes: ' + scenesResult.error.message;
			}
		} catch (e) {
			error = 'Failed to load data: ' + String(e);
		} finally {
			loading = false;
		}
	}

	function toggleScene(sceneId: string) {
		if (selectedSceneIds.has(sceneId)) {
			selectedSceneIds.delete(sceneId);
		} else {
			selectedSceneIds.add(sceneId);
		}
	}

	async function handleSubmit() {
		if (!selectedVersionId) {
			error = 'Please select a shader version';
			return;
		}

		if (selectedSceneIds.size === 0) {
			error = 'Please select at least one scene';
			return;
		}

		loading = true;
		error = null;

		const request: CreateJobRequest = {
			shader_version_id: selectedVersionId,
			scene_ids: Array.from(selectedSceneIds),
			profiles: profilesInput.trim() ? profilesInput.split(',').map((p) => p.trim()) : undefined,
			priority: priority || undefined
		};

		const result = await api.admin.createJob(request);

		if (result.isOk) {
			// Success! Reset and close
			resetForm();
			open = false;
			onJobCreated?.();
		} else {
			error = 'Failed to create job: ' + result.error.message;
		}

		loading = false;
	}

	function resetForm() {
		selectedShaderId = '';
		selectedVersionId = '';
		selectedSceneIds = new SvelteSet();
		profilesInput = '';
		priority = 0;
		error = null;
	}

	function handleOpenChange(newOpen: boolean) {
		open = newOpen;
		if (!newOpen) {
			resetForm();
		}
	}

	// Get versions for selected shader
	let availableVersions = $derived(
		selectedShaderId ? (shaderVersions.get(selectedShaderId) ?? []) : []
	);
</script>

<Dialog.Root {open} onOpenChange={handleOpenChange}>
	<Dialog.Trigger>
		<Button>
			<Plus class="mr-2 h-4 w-4" />
			Create Job
		</Button>
	</Dialog.Trigger>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-[600px]">
		<Dialog.Header>
			<Dialog.Title>Create New Job</Dialog.Title>
			<Dialog.Description>
				Create a capture job for a shader across multiple scenes
			</Dialog.Description>
		</Dialog.Header>

		{#if loading && shaders.length === 0}
			<div class="flex items-center justify-center py-8">
				<LoaderCircle class="h-8 w-8 animate-spin text-muted-foreground" />
			</div>
		{:else}
			<div class="grid gap-4 py-4">
				{#if error}
					<div
						class="flex items-start gap-2 rounded-lg border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
					>
						<CircleAlert class="h-4 w-4 shrink-0" />
						<div>{error}</div>
					</div>
				{/if}

				<!-- Shader Selection -->
				<div class="grid gap-2">
					<Label for="shader">Shader</Label>
					<select
						id="shader"
						class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
						bind:value={selectedShaderId}
						onchange={() => {
							selectedVersionId = '';
						}}
					>
						<option value="">Select a shader...</option>
						{#each shaders as shader (shader.id)}
							<option value={shader.id}>{shader.name}</option>
						{/each}
					</select>
				</div>

				<!-- Version Selection -->
				{#if selectedShaderId}
					<div class="grid gap-2">
						<Label for="version">Version</Label>
						<select
							id="version"
							class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
							bind:value={selectedVersionId}
						>
							<option value="">Select a version...</option>
							{#each availableVersions as version (version.id)}
								<option value={version.id}>{version.version}</option>
							{/each}
						</select>
					</div>
				{/if}

				<!-- Scene Selection -->
				<div class="grid gap-2">
					<Label>Scenes ({selectedSceneIds.size} selected)</Label>
					<div class="max-h-48 space-y-2 overflow-y-auto rounded-md border p-3">
						{#each scenes as scene (scene.id)}
							<label class="flex items-center gap-2 text-sm">
								<input
									type="checkbox"
									class="h-4 w-4 rounded border-gray-300"
									checked={selectedSceneIds.has(scene.id)}
									onchange={() => { toggleScene(scene.id); }}
								/>
								<span>{scene.name}</span>
								<span class="text-xs text-muted-foreground">({scene.slug})</span>
							</label>
						{/each}
					</div>
				</div>

				<!-- Profiles (Optional) -->
				<div class="grid gap-2">
					<Label for="profiles">Profiles (optional)</Label>
					<Input
						id="profiles"
						placeholder="e.g., High, Medium, Low (comma-separated)"
						bind:value={profilesInput}
					/>
					<p class="text-xs text-muted-foreground">Leave empty to capture all available profiles</p>
				</div>

				<!-- Priority (Optional) -->
				<div class="grid gap-2">
					<Label for="priority">Priority</Label>
					<Input id="priority" type="number" min="0" max="100" bind:value={priority} />
					<p class="text-xs text-muted-foreground">Higher priority jobs are processed first</p>
				</div>
			</div>

			<Dialog.Footer>
				<Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
				<Button onclick={handleSubmit} disabled={loading}>
					{#if loading}
						<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					Create Job
				</Button>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
