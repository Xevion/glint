import { createApiClient } from '$lib/api';
import type { SceneListItem, SceneVersion } from '$lib/bindings';
import { pick } from '$lib/utils';
import type { PageLoad } from './$types';

type TrimmedScene = Pick<
	SceneListItem,
	'id' | 'slug' | 'name' | 'description' | 'dimension' | 'image_url' | 'thumbhash' | 'capture_count'
> & {
	version: Pick<SceneVersion, 'time_of_day_ticks' | 'weather' | 'biome'>;
};

function trimScene(s: SceneListItem): TrimmedScene {
	return {
		...pick(s, [
			'id',
			'slug',
			'name',
			'description',
			'dimension',
			'image_url',
			'thumbhash',
			'capture_count'
		]),
		version: pick(s.version, ['time_of_day_ticks', 'weather', 'biome'])
	};
}

interface ScenesPageData {
	scenes: TrimmedScene[];
	error?: string;
}

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.scenes.list();

	return result.match<ScenesPageData>({
		Ok: (scenes) => ({
			scenes: scenes.map(trimScene)
		}),
		Err: (error) => ({
			scenes: [],
			error: error.message
		})
	});
};
