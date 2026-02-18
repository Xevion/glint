import { createApiClient } from '$lib/api';
import { ApiErrorType, pageError } from '$lib/api/errors';
import type {
	CaptureWithContext,
	ShaderAuthor,
	ShaderListItem,
	ShaderVersionDetail,
	ShaderVersionMetadata,
	ShaderVersionProfile,
	ShaderWithCaptures
} from '$lib/bindings';
import { pick } from '$lib/utils';
import type { PageLoad } from './$types';

export type ShaderDetailVersion = Pick<ShaderVersionDetail, 'id' | 'version' | 'capture_count'>;
export type ShaderDetailCapture = Pick<
	CaptureWithContext,
	| 'id'
	| 'image_url'
	| 'image_path'
	| 'thumbhash'
	| 'scene_name'
	| 'profile_display_name'
	| 'shader_version'
	| 'shader_name'
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
	| 'curseforge_id'
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

export interface CapturesPageData {
	items: ShaderDetailCapture[];
	total: number;
	page: number;
	pageSize: number;
}

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

/** Trim a CaptureWithContext to only the fields needed for the shader detail page. */
export function _trimCapture(c: CaptureWithContext): ShaderDetailCapture {
	return pick(c, [
		'id',
		'image_url',
		'image_path',
		'thumbhash',
		'scene_name',
		'profile_display_name',
		'shader_version',
		'shader_name'
	]);
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
			'curseforge_id',
			'source_url',
			'upstream_downloads',
			'view_count'
		]),
		authors: deduplicateAuthors(s.authors),
		versions: s.versions.map((v) => pick(v, ['id', 'version', 'capture_count'])),
		captures: s.captures.map(_trimCapture),
		profiles: s.profiles,
		metadata: s.metadata
	};
}

const CAPTURES_PAGE_SIZE = 24;

export const load: PageLoad = async ({ params, fetch, url }) => {
	const api = createApiClient(fetch);
	const result = await api.shaders.getShader(params.slug);

	let shader = result.match({
		Ok: (s) => s,
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				pageError(404, `Shader "${params.slug}" not found`);
			}
			return err.throw();
		}
	});

	// Default fetch returns captures for the latest version. If that version has
	// no captures but an older one does, re-fetch with the correct version so SSR
	// data matches the version the UI will select.
	let effectiveVersionId: string | undefined;
	if (shader.captures.length === 0) {
		const versionWithCaptures = shader.versions.find((v) => v.capture_count > 0);
		if (versionWithCaptures) {
			effectiveVersionId = versionWithCaptures.id;
			const refetch = await api.shaders.getShader(params.slug, {
				versionId: versionWithCaptures.id
			});
			if (refetch.isOk) {
				shader = refetch.value;
			}
		}
	}

	// Resolve version/profile from URL params or the effective version used above
	const versionId = url.searchParams.get('version_id') ?? effectiveVersionId;
	const profileId = url.searchParams.get('profile_id') ?? undefined;

	const trimmedShader = _trimShader(shader);

	// Captures list and similar shaders are independent — fetch in parallel
	const [capturesResult, listResult] = await Promise.all([
		api.shaders.listCaptures(shader.slug, {
			page: 1,
			pageSize: CAPTURES_PAGE_SIZE,
			versionId: versionId ?? undefined,
			profileId
		}),
		api.shaders.list({ pageSize: 30 })
	]);

	const capturesData: CapturesPageData = capturesResult.match({
		Ok: (p) => ({
			items: p.items.map(_trimCapture),
			total: p.total,
			page: p.page,
			pageSize: p.page_size
		}),
		Err: () => ({
			items: trimmedShader.captures,
			total: trimmedShader.captures.length,
			page: 1,
			pageSize: CAPTURES_PAGE_SIZE
		})
	});

	const similarShaders: ShaderListItem[] = listResult.match({
		Ok: (page) => page.items.filter((s) => s.slug !== params.slug),
		Err: () => []
	});

	return { shader: trimmedShader, capturesData, similarShaders };
};
