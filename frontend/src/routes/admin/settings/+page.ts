import { createGraphQLClient, graphql, query, type ResultOf } from '$lib/graphql';
import type { PageLoad } from './$types';

const AdminBackgroundsQuery = graphql(`
	query AdminBackgrounds {
		allBackgrounds {
			id
			imagePath
			thumbhash
			themeMode
			enabled
			sortOrder
			originalFilename
			width
			height
			fileSizeBytes
			contentType
			createdAt
			updatedAt
		}
	}
`);

export type AdminBackground = ResultOf<typeof AdminBackgroundsQuery>['allBackgrounds'][number];

export const load: PageLoad = async ({ fetch }) => {
	const client = createGraphQLClient(fetch);
	const result = await query(client, AdminBackgroundsQuery, {});

	return result.match({
		Ok: (data) => ({
			backgrounds: data.allBackgrounds,
			error: null as string | null
		}),
		Err: (err) => ({ backgrounds: [] as AdminBackground[], error: err.message })
	});
};
