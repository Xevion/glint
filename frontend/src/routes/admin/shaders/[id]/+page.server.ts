import { fail } from '@sveltejs/kit';
import { message, superValidate } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { createApiClient } from '$lib/api';
import type { UpdateShaderRequest } from '$lib/bindings';
import { shaderFormSchema } from './schema';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ request, fetch, params }) => {
		const form = await superValidate(request, zod4(shaderFormSchema));
		if (!form.valid) return fail(400, { form });

		const api = createApiClient(fetch);

		// Build the update request. Empty optional strings → undefined to clear.
		const data = form.data;
		const req: UpdateShaderRequest = {
			name: data.name,
			description: data.description || undefined,
			modrinth_id: data.modrinth_id || undefined,
			curseforge_id: data.curseforge_id || undefined,
			website_url: data.website_url || undefined,
			capture_enabled: data.capture_enabled,
			// null → empty string to clear the pin
			preferred_version_id: data.preferred_version_id ?? ''
		};

		const result = await api.admin.updateShader(params.id, req);

		return result.match({
			Ok: () => message(form, 'Shader updated successfully'),
			Err: (err) => message(form, err.message, { status: 400 })
		});
	}
};
