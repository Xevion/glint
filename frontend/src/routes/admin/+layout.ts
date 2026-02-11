import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ parent }) => {
	const { user } = await parent();

	const isAdmin = user?.role === 'admin';

	return { isAdmin };
};
