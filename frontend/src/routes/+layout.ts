import { createApiClient } from '$lib/api';
import type { Background, User } from '$lib/bindings';
import { pick } from '$lib/utils';
import type { LayoutLoad } from './$types';

type TrimmedBackground = Pick<
	Background,
	'id' | 'image_path' | 'thumbhash' | 'theme_mode' | 'width' | 'height'
>;

type TrimmedUser = Pick<User, 'role' | 'discord_id' | 'discord_username' | 'discord_avatar'>;

export const load: LayoutLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const [userResult, bgResult] = await Promise.all([api.user.me(), api.backgrounds.list()]);

	const user: TrimmedUser | null = userResult.match({
		Ok: (u) => pick(u, ['role', 'discord_id', 'discord_username', 'discord_avatar']),
		Err: () => null
	});

	const backgrounds: TrimmedBackground[] = bgResult.match({
		Ok: (bgs) =>
			bgs.map((b) => pick(b, ['id', 'image_path', 'thumbhash', 'theme_mode', 'width', 'height'])),
		Err: () => []
	});

	return { user, backgrounds };
};
