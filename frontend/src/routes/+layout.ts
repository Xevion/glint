import { createApiClient } from '$lib/api';
import type { User } from '$lib/bindings';
import { createGraphQLClient, graphql, query, type ResultOf } from '$lib/graphql';
import { pick } from '$lib/utils';
import type { LayoutLoad } from './$types';

type TrimmedUser = Pick<User, 'role' | 'discord_id' | 'discord_username' | 'discord_avatar'>;

const LayoutBackgroundsQuery = graphql(`
	query LayoutBackgrounds {
		backgrounds {
			id
			imagePath
			thumbhash
			themeMode
			enabled
			sortOrder
			width
			height
		}
	}
`);

type GqlBackground = ResultOf<typeof LayoutBackgroundsQuery>['backgrounds'][number];

/** Lowercase the GraphQL enum ("LIGHT" → "light") for BackgroundItem.themeMode */
function normalizeBackground(bg: GqlBackground) {
	return { ...bg, themeMode: bg.themeMode.toLowerCase() };
}

export const load: LayoutLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const client = createGraphQLClient(fetch);
	const [userResult, bgResult] = await Promise.all([
		api.user.me(),
		query(client, LayoutBackgroundsQuery, {})
	]);

	const user: TrimmedUser | null = userResult.match({
		Ok: (u) => pick(u, ['role', 'discord_id', 'discord_username', 'discord_avatar']),
		Err: () => null
	});

	const backgrounds = bgResult.match({
		Ok: (data) => data.backgrounds.map(normalizeBackground),
		Err: () => [] as ReturnType<typeof normalizeBackground>[]
	});

	return { user, backgrounds };
};
