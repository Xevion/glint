import { createApiClient, ApiErrorType } from '$lib/api';
import { pageError } from '$lib/api/errors';
import { superValidate } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { shaderFormSchema } from './schema';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getShader(params.id);

	return result.match({
		Ok: async (shader) => ({
			shader,
			form: await superValidate(
				{
					name: shader.name,
					description: shader.description ?? '',
					modrinth_id: shader.modrinth_id ?? '',
					curseforge_id: shader.curseforge_id ?? '',
					website_url: shader.website_url ?? '',
					capture_enabled: shader.capture_enabled,
					preferred_version_id: shader.preferred_version_id ?? null
				},
				zod4(shaderFormSchema)
			)
		}),
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Shader not found');
			return err.throw();
		}
	});
};
