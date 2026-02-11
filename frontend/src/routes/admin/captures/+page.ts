import { createApiClient } from '$lib/api';
import type { CaptureWithContext, SceneWithWorld, Shader } from '$lib/bindings';
import type { PageLoad } from './$types';

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

	const shaders = shadersRes.match({
		Ok: (v) => v,
		Err: () => [] as Shader[]
	});

	const scenes = scenesRes.match({
		Ok: (v) => v,
		Err: () => [] as SceneWithWorld[]
	});

	return result.match({
		Ok: (data) => ({
			captures: data.items,
			totalCount: data.total,
			page: data.page,
			pageSize: data.pageSize,
			error: null as string | null,
			filters: { shader, scene, status, runId },
			shaders,
			scenes
		}),
		Err: (err) => ({
			captures: [] as CaptureWithContext[],
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
