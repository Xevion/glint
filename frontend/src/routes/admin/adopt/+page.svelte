<script lang="ts">
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { Search, LoaderCircle, Download, CircleAlert } from '@lucide/svelte';
import { api } from '$lib/api';
import type { ShaderSearchResult } from '$lib/bindings';
import AdoptShaderDialog from '$lib/components/adopt-shader-dialog.svelte';

let query = $state('');
let results = $state<ShaderSearchResult[]>([]);
let totalModrinth = $state(0);
let totalCurseforge = $state<number | null>(null);
let searching = $state(false);
let searched = $state(false);
let error = $state<string | null>(null);

let adoptDialogOpen = $state(false);
let adoptUrl = $state<string | undefined>(undefined);

async function handleSearch() {
	const q = query.trim();
	if (!q) return;

	searching = true;
	error = null;

	const result = await api.adopt.search(q, 20, 0);
	if (result.isOk) {
		results = result.value.results;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		error = result.error.message;
		results = [];
	}
	searching = false;
	searched = true;
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter') {
		void handleSearch();
	}
}

function adoptFromResult(result: ShaderSearchResult) {
	adoptUrl = result.platform_url;
	adoptDialogOpen = true;
}

function handleShaderAdopted() {
	adoptUrl = undefined;
	if (searched) void handleSearch();
}

function formatNumber(n: number): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
	return n.toLocaleString();
}
</script>

<div class="space-y-4">
	<header>
		<h1 class="text-2xl font-semibold">Adopt Shaders</h1>
		<p class="text-sm text-muted-foreground mt-1">
			Search for shaders on Modrinth and CurseForge to adopt into Glint.
		</p>
	</header>

	<div class="flex gap-2">
		<Input
			placeholder="Search for shaders..."
			bind:value={query}
			onkeydown={handleKeydown}
			disabled={searching}
			class="max-w-md"
		/>
		<Button onclick={handleSearch} disabled={searching || !query.trim()}>
			{#if searching}
				<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
			{:else}
				<Search class="mr-2 h-4 w-4" />
			{/if}
			Search
		</Button>
	</div>

	{#if error}
		<div
			class="flex items-start gap-2 rounded-lg border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
		>
			<CircleAlert class="h-4 w-4 shrink-0 mt-0.5" />
			<div>{error}</div>
		</div>
	{/if}

	{#if searched && !searching && results.length === 0 && !error}
		<p class="text-muted-foreground py-8 text-center">No shaders found for "{query}".</p>
	{:else if results.length > 0}
		<div class="text-xs text-muted-foreground">
			{totalModrinth} Modrinth results{totalCurseforge !== null
				? `, ${totalCurseforge} CurseForge results`
				: ''}
		</div>

		<!-- Desktop table -->
		<div class="hidden md:block rounded-md border">
			<table class="w-full text-sm">
				<thead>
					<tr class="border-b bg-muted/50">
						<th class="p-3 text-left font-medium w-10"></th>
						<th class="p-3 text-left font-medium">Name</th>
						<th class="p-3 text-left font-medium">Platform</th>
						<th class="p-3 text-left font-medium">Author</th>
						<th class="p-3 text-right font-medium">Downloads</th>
						<th class="p-3 text-right font-medium w-24"></th>
					</tr>
				</thead>
				<tbody>
					{#each results as result (result.platform + ':' + result.platform_id)}
						<tr class="border-b last:border-b-0 hover:bg-muted/30 transition-colors">
							<td class="p-3">
								{#if result.icon_url}
									<img
										src={result.icon_url}
										alt=""
										class="h-8 w-8 rounded object-cover"
									/>
								{:else}
									<div
										class="h-8 w-8 rounded bg-muted flex items-center justify-center"
									>
										<Download class="h-4 w-4 text-muted-foreground" />
									</div>
								{/if}
							</td>
							<td class="p-3">
								<div class="font-medium">{result.name}</div>
								<div class="text-xs text-muted-foreground line-clamp-1">
									{result.description}
								</div>
							</td>
							<td class="p-3">
								<span
									class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium
										{result.platform === 'modrinth'
										? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
										: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'}"
								>
									{result.platform === 'modrinth' ? 'Modrinth' : 'CurseForge'}
								</span>
							</td>
							<td class="p-3 text-muted-foreground">{result.author}</td>
							<td class="p-3 text-right tabular-nums">
								{formatNumber(result.downloads)}
							</td>
							<td class="p-3 text-right">
								<Button
									size="sm"
									variant="outline"
									onclick={() => adoptFromResult(result)}
								>
									Adopt
								</Button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- Mobile cards -->
		<div class="md:hidden space-y-2">
			{#each results as result (result.platform + ':' + result.platform_id)}
				<div class="rounded-lg border p-3 space-y-2">
					<div class="flex items-start gap-3">
						{#if result.icon_url}
							<img
								src={result.icon_url}
								alt=""
								class="h-10 w-10 rounded object-cover shrink-0"
							/>
						{/if}
						<div class="min-w-0 flex-1">
							<div class="font-medium">{result.name}</div>
							<div class="text-xs text-muted-foreground line-clamp-1">
								{result.description}
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between text-xs">
						<div class="flex items-center gap-2">
							<span
								class="inline-flex items-center rounded-full px-2 py-0.5 font-medium
									{result.platform === 'modrinth'
									? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
									: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'}"
							>
								{result.platform === 'modrinth' ? 'Modrinth' : 'CurseForge'}
							</span>
							<span class="text-muted-foreground">{result.author}</span>
							<span class="text-muted-foreground tabular-nums">
								{formatNumber(result.downloads)}
							</span>
						</div>
						<Button size="sm" variant="outline" onclick={() => adoptFromResult(result)}>
							Adopt
						</Button>
					</div>
				</div>
			{/each}
		</div>
	{:else if !searched}
		<div class="py-12 text-center text-muted-foreground">
			Search for shaders on Modrinth and CurseForge to adopt them.
		</div>
	{/if}
</div>

<AdoptShaderDialog
	bind:open={adoptDialogOpen}
	initialUrl={adoptUrl}
	onShaderAdopted={handleShaderAdopted}
/>
