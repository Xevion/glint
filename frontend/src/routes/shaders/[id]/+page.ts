import { createApiClient } from '$lib/api';
import { ApiErrorType } from '$lib/api/errors';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.shaders.getShader(params.id);

	let shader = result.match({
		Ok: (s) => s,
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				error(404, { message: `Shader "${params.id}" not found` });
			}
			return err.throw();
		}
	});

	// Default fetch returns captures for the latest version. If that version has
	// no captures but an older one does, re-fetch with the correct version so SSR
	// data matches the version the UI will select.
	if (shader.captures.length === 0) {
		const versionWithCaptures = shader.versions.find((v) => v.capture_count > 0);
		if (versionWithCaptures) {
			const refetch = await api.shaders.getShader(params.id, {
				versionId: versionWithCaptures.id
			});
			if (refetch.isOk) {
				shader = refetch.value;
			}
		}
	}

	return { shader };
};
