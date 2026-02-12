import { createApiClient } from '$lib/api';
import { ApiErrorType } from '$lib/api/errors';
import type {
	CaptureWithContext,
	ShaderAuthor,
	ShaderVersionDetail,
	ShaderVersionMetadata,
	ShaderVersionProfile,
	ShaderWithCaptures
} from '$lib/bindings';
import { pick } from '$lib/utils';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export type ShaderDetailVersion = Pick<ShaderVersionDetail, 'id' | 'version' | 'capture_count'>;
export type ShaderDetailCapture = Pick<
	CaptureWithContext,
	'id' | 'image_url' | 'thumbhash' | 'scene_name' | 'profile_name' | 'shader_version' | 'shader_name'
>;
export type ShaderDetailAuthor = Pick<ShaderAuthor, 'name' | 'url'>;
export type ShaderDetail = Pick<
	ShaderWithCaptures,
	| 'id'
	| 'name'
	| 'slug'
	| 'description'
	| 'icon_url'
	| 'website_url'
	| 'modrinth_id'
	| 'source_url'
	| 'upstream_downloads'
	| 'view_count'
> & {
	authors: ShaderDetailAuthor[];
	versions: ShaderDetailVersion[];
	captures: ShaderDetailCapture[];
	profiles: ShaderVersionProfile[];
	metadata?: ShaderVersionMetadata;
};

/** Deduplicate authors by name, preferring entries that have a URL. */
function deduplicateAuthors(authors: ShaderAuthor[]): ShaderDetailAuthor[] {
	const byName = new Map<string, ShaderDetailAuthor>();
	for (const a of authors) {
		const existing = byName.get(a.name);
		if (!existing || (!existing.url && a.url)) {
			byName.set(a.name, { name: a.name, url: a.url });
		}
	}
	return [...byName.values()];
}

export function _trimShader(s: ShaderWithCaptures): ShaderDetail {
	return {
		...pick(s, [
			'id',
			'name',
			'slug',
			'description',
			'icon_url',
			'website_url',
			'modrinth_id',
			'source_url',
			'upstream_downloads',
			'view_count'
		]),
		authors: deduplicateAuthors(s.authors),
		versions: s.versions.map((v) => pick(v, ['id', 'version', 'capture_count'])),
		captures: s.captures.map((c) =>
			pick(c, [
				'id',
				'image_url',
				'thumbhash',
				'scene_name',
				'profile_name',
				'shader_version',
				'shader_name'
			])
		),
		profiles: s.profiles,
		metadata: s.metadata
	};
}

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.shaders.getShader(params.slug);

	let shader = result.match({
		Ok: (s) => s,
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				error(404, { message: `Shader "${params.slug}" not found` });
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
			const refetch = await api.shaders.getShader(params.slug, {
				versionId: versionWithCaptures.id
			});
			if (refetch.isOk) {
				shader = refetch.value;
			}
		}
	}

	return { shader: _trimShader(shader) };
};
