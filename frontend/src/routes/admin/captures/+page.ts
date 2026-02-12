import { createApiClient } from '$lib/api';
import type { CaptureWithContext, SceneListItem, ShaderListItem } from '$lib/bindings';
import { pick } from '$lib/utils';
import type { PageLoad } from './$types';

type FilterOption = Pick<ShaderListItem, 'id' | 'slug' | 'name'>;
type SceneFilterOption = Pick<SceneListItem, 'id' | 'name'>;

export const load: PageLoad = async ({ fetch, url }) => {
	const api = createApiClient(fetch);

	const page = Number(url.searchParams.get('page') ?? '1');
	const pageSize = Number(url.searchParams.get('pageSize') ?? '50');
	const shader = url.searchParams.get('shader') ?? undefined;
	const scene = url.searchParams.get('scene') ?? undefined;
	const status = url.searchParams.get('status') ?? undefined;
	const runId = url.searchParams.get('runId') ?? undefined;

	const [result, shadersRes, scenesRes] = await Promise.all([
		api.admin.listCaptures({ page, pageSize, shader, scene, status, runId }),
		api.admin.listShaders(),
		api.admin.listScenes()
	]);

	const shaders: FilterOption[] = shadersRes.match({
		Ok: (v) => v.items.map((s) => pick(s, ['id', 'slug', 'name'])),
		Err: () => []
	});

	const scenes: SceneFilterOption[] = scenesRes.match({
		Ok: (v) => v.map((s) => pick(s, ['id', 'name'])),
		Err: () => []
	});

	interface CapturesPageData {
		captures: CaptureWithContext[];
		totalCount: number;
		page: number;
		pageSize: number;
		error: string | null;
		filters: { shader?: string; scene?: string; status?: string; runId?: string };
		shaders: FilterOption[];
		scenes: SceneFilterOption[];
	}

	return result.match<CapturesPageData>({
		Ok: (data) => ({
			captures: data.items,
			totalCount: data.total,
			page: data.page,
			pageSize: data.page_size,
			error: null,
			filters: { shader, scene, status, runId },
			shaders,
			scenes
		}),
		Err: (err) => ({
			captures: [],
			totalCount: 0,
			page: 1,
			pageSize: 50,
			error: err.message,
			filters: { shader, scene, status, runId },
			shaders,
			scenes
		})
	});
};
