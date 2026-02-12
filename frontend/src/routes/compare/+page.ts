import { createApiClient } from '$lib/api';
import type { CaptureWithContext, SceneListItem } from '$lib/bindings';
import { pick } from '$lib/utils';
import type { PageLoad } from './$types';

type CompareScene = Pick<SceneListItem, 'slug' | 'name' | 'capture_count'>;
type CompareCapture = Pick<
	CaptureWithContext,
	'thumbhash' | 'shader_name' | 'shader_version' | 'shader_author' | 'profile_name'
> & { image_url: string };

interface ComparePageData {
	scenes: CompareScene[];
	captures: CompareCapture[];
	selectedSceneSlug: string | null;
	error: string | null;
}

export const load: PageLoad = async ({ fetch, url }): Promise<ComparePageData> => {
	const api = createApiClient(fetch);
	const sceneSlug = url.searchParams.get('scene') ?? undefined;

	const scenesResult = await api.scenes.list();

	const scenes: CompareScene[] = scenesResult.match({
		Ok: (v) => v.map((s) => pick(s, ['slug', 'name', 'capture_count'])),
		Err: () => []
	});

	// Pick the target scene: explicit slug, or first scene with captures
	const targetSlug = sceneSlug ?? scenes.find((s) => s.capture_count > 0)?.slug ?? scenes[0]?.slug;

	if (!targetSlug) {
		return {
			scenes,
			captures: [],
			selectedSceneSlug: null,
			error: null
		};
	}

	const sceneResult = await api.scenes.getBySlug(targetSlug);

	return sceneResult.match({
		Ok: (data) => {
			const captures: CompareCapture[] = (data[0]?.captures ?? [])
				.filter((c): c is typeof c & { image_url: string } => !!c.image_url)
				.map((c) => ({
					...pick(c, ['thumbhash', 'shader_name', 'shader_version', 'shader_author', 'profile_name']),
					image_url: c.image_url
				}));
			return {
				scenes,
				captures,
				selectedSceneSlug: targetSlug,
				error: null
			};
		},
		Err: (err): ComparePageData => ({
			scenes,
			captures: [],
			selectedSceneSlug: targetSlug,
			error: err.message
		})
	});
};
