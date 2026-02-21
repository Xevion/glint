import { createApiClient } from '$lib/api';
import type { User } from '$lib/bindings';
import { type ResultOf, createGraphQLClient, graphql, query } from '$lib/graphql';
import { pick } from '$lib/utils';
import type { LayoutServerLoad } from './$types';

type TrimmedUser = Pick<User, 'role' | 'discord_id' | 'discord_username' | 'discord_avatar'>;

const LayoutBackgroundsQuery = graphql(`
	query LayoutBackgrounds {
		backgrounds {
			id
			imagePath
			thumbhash
			themeMode
			width
			height
		}
	}
`);

type GqlBackground = ResultOf<typeof LayoutBackgroundsQuery>['backgrounds'][number];

export interface BackgroundItem {
	id: string;
	imagePath: string;
	thumbhash?: string | null;
	width?: number | null;
	height?: number | null;
}

function splitBackgrounds(backgrounds: GqlBackground[]): {
	light: BackgroundItem[];
	dark: BackgroundItem[];
} {
	const toItem = (bg: GqlBackground): BackgroundItem => ({
		id: bg.id,
		imagePath: bg.imagePath,
		thumbhash: bg.thumbhash,
		width: bg.width,
		height: bg.height
	});
	const mode = (bg: GqlBackground) => bg.themeMode.toLowerCase();
	return {
		light: backgrounds
			.filter((b) => {
				const m = mode(b);
				return m === 'light' || m === 'both';
			})
			.map(toItem),
		dark: backgrounds
			.filter((b) => {
				const m = mode(b);
				return m === 'dark' || m === 'both';
			})
			.map(toItem)
	};
}

export const load: LayoutServerLoad = async ({ fetch, depends }) => {
	depends('glint:user');
	depends('glint:backgrounds');
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

	const { light, dark } = bgResult.match({
		Ok: (data) => splitBackgrounds(data.backgrounds),
		Err: () => ({ light: [] as BackgroundItem[], dark: [] as BackgroundItem[] })
	});

	return { user, lightBackgrounds: light, darkBackgrounds: dark };
};
