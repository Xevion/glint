<script lang="ts">
import { fly, fade, scale } from 'svelte/transition';
import { SvelteMap } from 'svelte/reactivity';
import { resolve } from '$app/paths';
import { Button } from '$lib/components/ui/button';
import type { CaptureWithContext, ShaderWithCaptures } from '$lib/bindings';

interface Props {
	data: { shader: ShaderWithCaptures };
}

let { data }: Props = $props();
const shader = $derived(data.shader);
const captures = $derived(shader.captures);

// Selected capture for the main preview (state so user can change it)
let selectedCapture = $state<(typeof captures)[0] | null>(null);

// Initialize selected capture when captures change
$effect(() => {
	if (captures.length > 0 && !selectedCapture) {
		selectedCapture = captures[0];
	}
});

// Get the latest version (assumes versions are sorted newest first)
const latestVersion = $derived(shader.versions[0]);

// Group captures by scene for thumbnail selector
const capturesByScene = $derived.by(() => {
	const map = new SvelteMap<string, CaptureWithContext>();
	for (const capture of captures) {
		if (!map.has(capture.scene_id)) {
			map.set(capture.scene_id, capture);
		}
	}
	return Array.from(map.values());
});
</script>

{#if shader}
	{#key shader.id}
		<div class="container mx-auto px-4 py-8">
			<!-- Breadcrumb -->
			<nav
				in:fly|local={{ y: -10, duration: 400 }}
				class="mb-6 flex items-center gap-2 text-sm text-muted-foreground"
			>
				<a href={resolve('/shaders')} class="transition-colors hover:text-foreground">Shaders</a>
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
				</svg>
				<span class="font-medium text-foreground">{shader.name}</span>
			</nav>

			<!-- Hero: Preview + Info -->
			<div
				in:fly|local={{ y: 10, duration: 400, delay: 100 }}
				class="mb-8 grid gap-6 lg:grid-cols-3"
			>
				<!-- Main Preview Area -->
				<div class="space-y-4 lg:col-span-2">
					<!-- Main Image -->
					<div
						class="shadow-theme-lg relative aspect-video w-full overflow-hidden rounded-xl bg-card"
					>
						{#if selectedCapture}
							<img
								src={selectedCapture.screenshot_url}
								alt="{shader.name} capture"
								class="h-full w-full object-cover"
							/>

							<!-- Overlay with capture info -->
							<div
								class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent"
							>
								<div class="absolute right-0 bottom-0 left-0 p-4">
									<div class="flex items-end justify-between">
										<div>
											<h3 class="mb-2 text-xl font-bold text-white">{shader.name}</h3>
											<div class="flex items-center gap-2">
												{#if selectedCapture.profile}
													<span class="rounded bg-primary px-2 py-0.5 text-xs font-bold text-white">
														{selectedCapture.profile}
													</span>
												{/if}
												{#if selectedCapture.shader_version}
													<span class="rounded bg-muted px-2 py-0.5 text-xs font-bold">
														v{selectedCapture.shader_version}
													</span>
												{/if}
											</div>
										</div>
									</div>
								</div>
							</div>
						{:else}
							<div class="flex h-full items-center justify-center text-muted-foreground">
								<div class="text-center">
									<svg
										class="mx-auto mb-4 h-16 w-16"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="1.5"
											d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
										/>
									</svg>
									<p class="text-sm">No captures available yet</p>
								</div>
							</div>
						{/if}
					</div>

					<!-- Scene Thumbnails -->
					{#if capturesByScene.length > 0}
						<div class="grid grid-cols-4 gap-2">
							{#each capturesByScene as capture (capture.id)}
								<button
									type="button"
									onclick={() => (selectedCapture = capture)}
									class="aspect-video overflow-hidden rounded-lg border-2 transition-all hover:border-primary {selectedCapture?.id ===
									capture.id
										? 'border-primary'
										: 'border-transparent'}"
								>
									{#if capture.screenshot_url}
										<img
											src={capture.screenshot_url}
											alt="Scene thumbnail"
											class="h-full w-full object-cover"
										/>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>

				<!-- Shader Info Sidebar -->
				<div class="space-y-4">
					<!-- Basic Info -->
					<div class="rounded-xl border border-border bg-card p-6">
						<h2 class="mb-4 text-2xl font-bold text-card-foreground">{shader.name}</h2>
						{#if shader.description}
							<p class="mb-4 text-sm text-muted-foreground">{shader.description}</p>
						{/if}

						<!-- Version Info -->
						{#if latestVersion}
							<div class="mb-4 rounded-lg bg-muted p-3">
								<div class="text-xs font-medium text-muted-foreground">Latest Version</div>
								<div class="text-lg font-bold text-foreground">{latestVersion.version}</div>
							</div>
						{/if}

						<!-- Stats -->
						<div>
							<div class="text-2xl font-bold text-foreground">{shader.versions.length}</div>
							<div class="text-xs text-muted-foreground">Versions</div>
						</div>

						<!-- Links -->
						<div class="mt-4 flex flex-col gap-2">
							{#if shader.website_url}
								<Button href={shader.website_url} variant="outline" class="w-full">
									Visit Website
								</Button>
							{/if}
							{#if shader.modrinth_id}
								<Button
									href="https://modrinth.com/shader/{shader.modrinth_id}"
									variant="outline"
									class="w-full"
								>
									View on Modrinth
								</Button>
							{/if}
						</div>
					</div>

					<!-- Versions -->
					{#if shader.versions.length > 0}
						<div class="rounded-xl border border-border bg-card p-6">
							<h3 class="mb-3 text-lg font-semibold text-card-foreground">Versions</h3>
							<div class="space-y-2">
								{#each shader.versions.slice(0, 5) as version (version.id)}
									<div class="flex items-center justify-between text-sm">
										<span class="font-mono text-foreground">{version.version}</span>
										{#if version.supported_profiles}
											{@const profiles = (() => {
												try {
													const parsed: unknown = JSON.parse(version.supported_profiles);
													return Array.isArray(parsed) ? (parsed as string[]) : [];
												} catch {
													return [];
												}
											})()}
											{#if profiles.length > 0}
												<span class="text-xs text-muted-foreground">
													{profiles.length} profiles
												</span>
											{/if}
										{/if}
									</div>
								{/each}
								{#if shader.versions.length > 5}
									<div class="pt-2 text-xs text-muted-foreground">
										+ {shader.versions.length - 5} more
									</div>
								{/if}
							</div>
						</div>
					{/if}
				</div>
			</div>

			<!-- Captures Grid -->
			{#if captures.length > 0}
				<div in:fade|local={{ duration: 300, delay: 200 }} class="mb-8">
					<h2 class="mb-4 text-2xl font-bold text-foreground">All Captures</h2>
					<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
						{#each captures as capture, i (capture.id)}
							<button
								type="button"
								onclick={() => (selectedCapture = capture)}
								in:scale|local={{ duration: 350, delay: Math.min(i * 50, 400) + 250, start: 0.95 }}
								class="group relative aspect-video overflow-hidden rounded-xl bg-card transition-transform hover:scale-[1.02]"
							>
								{#if capture.screenshot_url}
									<img
										src={capture.screenshot_url}
										alt="Capture"
										class="h-full w-full object-cover"
									/>
									<div
										class="absolute inset-0 bg-gradient-to-t from-black/60 to-transparent opacity-0 transition-opacity group-hover:opacity-100"
									>
										<div class="absolute right-0 bottom-0 left-0 p-3">
											<div class="flex items-center gap-2">
												{#if capture.profile}
													<span class="rounded bg-primary px-2 py-0.5 text-xs font-bold text-white">
														{capture.profile}
													</span>
												{/if}
												{#if capture.shader_version}
													<span
														class="rounded bg-white/20 px-2 py-0.5 text-xs font-bold text-white"
													>
														v{capture.shader_version}
													</span>
												{/if}
											</div>
										</div>
									</div>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			{:else}
				<div
					in:fade|local={{ duration: 300, delay: 200 }}
					class="mb-8 rounded-xl border border-border bg-card p-12 text-center"
				>
					<svg
						class="mx-auto mb-4 h-16 w-16 text-muted-foreground/30"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="1.5"
							d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
						/>
					</svg>
					<h3 class="mb-2 text-lg font-semibold text-card-foreground">No Captures Yet</h3>
					<p class="text-sm text-muted-foreground">Captures for this shader are being generated.</p>
				</div>
			{/if}
		</div>
	{/key}
{/if}
