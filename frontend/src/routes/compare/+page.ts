import { createApiClient } from '$lib/api';
import type { CaptureWithContext, SceneListItem } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
	const api = createApiClient(fetch);
	const sceneSlug = url.searchParams.get('scene') ?? undefined;

	const scenesResult = await api.scenes.list();

	const scenes = scenesResult.match({
		Ok: (v) => v,
		Err: () => [] as SceneListItem[]
	});

	// Pick the target scene: explicit slug, or first scene with captures
	const targetSlug = sceneSlug ?? scenes.find((s) => s.capture_count > 0)?.slug ?? scenes[0]?.slug;

	if (!targetSlug) {
		return {
			scenes,
			captures: [] as CaptureWithContext[],
			selectedSceneSlug: null as string | null,
			error: null as string | null
		};
	}

	const sceneResult = await api.scenes.getBySlug(targetSlug);

	return sceneResult.match({
		Ok: (data) => {
			const captures = (data[0]?.captures ?? []).filter((c) => c.image_url);
			return {
				scenes,
				captures,
				selectedSceneSlug: targetSlug as string | null,
				error: null as string | null
			};
		},
		Err: (err) => ({
			scenes,
			captures: [] as CaptureWithContext[],
			selectedSceneSlug: targetSlug as string | null,
			error: err.message
		})
	});
};
